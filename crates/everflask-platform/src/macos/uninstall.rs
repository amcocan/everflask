//! Removes everything EverFlask leaves on a Mac.
//!
//! MacOS has no uninstall hook: dragging an app to the Trash runs no code.
//! So EverFlask offers "Uninstall EverFlask..." in its menu, and
//! `packaging/macos/uninstall.sh` does the same job if the app is already
//! gone. Keep the two in sync.
//!
//! Every path and key removed is derived from the bundle identifier or the
//! app's own name; nothing here takes outside input.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use objc2::AnyThread;
use objc2_foundation::{NSBundle, NSFileManager, NSHomeDirectory, NSString, NSUserDefaults};

/// Must match `CFBundleIdentifier` in `packaging/macos/Info.plist`.
pub const BUNDLE_ID: &str = "io.github.amcocan.everflask";

/// Where MacOS records that it has shown an app the Accessibility warning.
const AUTH_WARNING_DOMAIN: &str = "com.apple.universalaccessAuthWarning";

/// Privacy services Simulate activity can be granted under.
const TCC_SERVICES: [&str; 2] = ["Accessibility", "PostEvent"];

/// What an uninstall step did, for showing to the user.
#[derive(Debug, Default)]
pub struct Report {
    pub removed: Vec<String>,
    pub failed: Vec<String>,
}

/// Everything except the app bundle itself. Run this **before** trashing the
/// app: `tccutil` can only reset permissions for an app MacOS can still find.
pub fn remove_artifacts(bundle_id: &str) -> Report {
    let mut report = Report::default();
    reset_permissions(bundle_id, &mut report);
    remove_auth_warning_keys(AUTH_WARNING_DOMAIN, bundle_id, &mut report);
    remove_support_dirs(bundle_id, &mut report);
    remove_preferences(bundle_id, &mut report);
    report
}

fn reset_permissions(bundle_id: &str, report: &mut Report) {
    for service in TCC_SERVICES {
        // Absolute path and fixed arguments: no shell, no PATH lookup.
        match Command::new("/usr/bin/tccutil")
            .args(["reset", service, bundle_id])
            .output()
        {
            Ok(out) if out.status.success() => {
                report.removed.push(format!("{service} permission"));
            }
            Ok(out) => report.failed.push(format!(
                "{service} permission: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => report.failed.push(format!("{service} permission: {e}")),
        }
    }
}

/// Deletes the app's preferences domain and its plist file. Safe to call
/// more than once; call it again just before exiting in case AppKit wrote
/// something in the meantime.
pub fn remove_preferences(bundle_id: &str, report: &mut Report) {
    // Through cfprefsd first, so its cache doesn't write the file back.
    NSUserDefaults::standardUserDefaults()
        .removePersistentDomainForName(&NSString::from_str(bundle_id));
    let plist = home().join(format!("Library/Preferences/{bundle_id}.plist"));
    remove_path(plist, report);
}

/// Removes only our entries from MacOS's "warning shown" list. Keys look like
/// `1::<bundle id>` or `1::<path to app or executable>`.
fn remove_auth_warning_keys(domain: &str, bundle_id: &str, report: &mut Report) {
    let name = NSString::from_str(domain);
    let Some(suite) = NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&name))
    else {
        return;
    };
    let Some(entries) = suite.persistentDomainForName(&name) else {
        return;
    };
    for key in &entries.allKeys() {
        let text = key.to_string();
        if is_our_auth_warning_key(&text, bundle_id) {
            suite.removeObjectForKey(&key);
            report
                .removed
                .push(format!("permission warning record ({text})"));
        }
    }
}

fn is_our_auth_warning_key(key: &str, bundle_id: &str) -> bool {
    let Some((_, target)) = key.split_once("::") else {
        return false;
    };
    let target = target.to_ascii_lowercase();
    target == bundle_id
        || target.ends_with("/everflask.app")
        || target.ends_with("/everflask.app/contents/macos/everflask")
}

/// Standard per-app folders. EverFlask doesn't create these today, but
/// AppKit can, and future features might.
fn remove_support_dirs(bundle_id: &str, report: &mut Report) {
    for rel in [
        format!("Library/Caches/{bundle_id}"),
        format!("Library/HTTPStorages/{bundle_id}"),
        format!("Library/Application Support/{bundle_id}"),
        format!("Library/Saved Application State/{bundle_id}.savedState"),
    ] {
        remove_path(home().join(rel), report);
    }
}

fn remove_path(path: PathBuf, report: &mut Report) {
    // symlink_metadata: a symlink is removed as a link, never followed.
    let Ok(meta) = fs::symlink_metadata(&path) else {
        return;
    };
    let result = if meta.is_dir() {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_file(&path)
    };
    match result {
        Ok(()) => report.removed.push(path.display().to_string()),
        Err(e) => report.failed.push(format!("{}: {e}", path.display())),
    }
}

fn home() -> PathBuf {
    PathBuf::from(NSHomeDirectory().to_string())
}

/// Moves the running app bundle to the Trash (recoverable, never deleted
/// outright). Refuses unless we're running from an `.app` with our bundle
/// identifier, so a dev build can't trash something unexpected.
pub fn trash_running_bundle(bundle_id: &str) -> Result<PathBuf, String> {
    let bundle = NSBundle::mainBundle();
    if bundle
        .bundleIdentifier()
        .map(|id| id.to_string())
        .as_deref()
        != Some(bundle_id)
    {
        return Err("not running from the EverFlask app bundle".into());
    }
    let url = bundle.bundleURL();
    let path = PathBuf::from(
        url.path()
            .map(|p| p.to_string())
            .ok_or("app bundle has no path")?,
    );
    if path.extension().and_then(|e| e.to_str()) != Some("app") {
        return Err(format!("{} is not an app bundle", path.display()));
    }
    NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, None)
        .map_err(|e| e.localizedDescription().to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "io.github.amcocan.everflask";

    #[test]
    fn matches_only_our_warning_keys() {
        assert!(is_our_auth_warning_key(
            "1::io.github.amcocan.everflask",
            ID
        ));
        assert!(is_our_auth_warning_key(
            "1::/Applications/EverFlask.app",
            ID
        ));
        assert!(is_our_auth_warning_key(
            "1::/Users/x/Everflask/target/debug/Everflask.app/Contents/MacOS/everflask",
            ID
        ));
        assert!(!is_our_auth_warning_key("1::com.example.other", ID));
        assert!(!is_our_auth_warning_key(
            "1::io.github.amcocan.everflask.helper",
            ID
        ));
        assert!(!is_our_auth_warning_key(
            "1::/Applications/NotEverflask.app.bak",
            ID
        ));
        assert!(!is_our_auth_warning_key("io.github.amcocan.everflask", ID));
    }

    /// Unique throwaway domain so tests never touch real preferences.
    fn scratch_domain(tag: &str) -> String {
        format!("{ID}.test-{tag}-{}", std::process::id())
    }

    #[test]
    fn removes_only_our_keys_from_warning_domain() {
        let domain = scratch_domain("authwarn");
        let name = NSString::from_str(&domain);
        let suite =
            NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&name)).expect("suite");
        for key in [
            "1::io.github.amcocan.everflask",
            "1::/Applications/EverFlask.app",
            "1::com.example.other",
        ] {
            suite.setBool_forKey(true, &NSString::from_str(key));
        }

        let mut report = Report::default();
        remove_auth_warning_keys(&domain, ID, &mut report);

        let left: Vec<String> = suite
            .persistentDomainForName(&name)
            .map(|d| d.allKeys().iter().map(|k| k.to_string()).collect())
            .unwrap_or_default();
        assert_eq!(left, ["1::com.example.other"]);
        assert_eq!(report.removed.len(), 2);

        let mut cleanup = Report::default();
        remove_preferences(&domain, &mut cleanup);
    }

    #[test]
    fn removes_preferences_domain_and_file() {
        let domain = scratch_domain("prefs");
        let name = NSString::from_str(&domain);
        let suite =
            NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&name)).expect("suite");
        suite.setBool_forKey(true, &NSString::from_str("probe"));
        suite.synchronize();

        let mut report = Report::default();
        remove_preferences(&domain, &mut report);

        assert!(NSUserDefaults::standardUserDefaults()
            .persistentDomainForName(&name)
            .is_none());
        let plist = home().join(format!("Library/Preferences/{domain}.plist"));
        assert!(!plist.exists(), "{} still exists", plist.display());
        assert!(report.failed.is_empty(), "{:?}", report.failed);
    }

    #[test]
    fn permission_reset_for_unknown_app_fails_gracefully() {
        let mut report = Report::default();
        reset_permissions(&scratch_domain("tcc"), &mut report);
        assert!(report.removed.is_empty());
        assert_eq!(report.failed.len(), TCC_SERVICES.len());
    }

    #[test]
    fn bundle_id_matches_info_plist() {
        let plist = include_str!("../../../../packaging/macos/Info.plist");
        let expected = format!("<key>CFBundleIdentifier</key>\n    <string>{BUNDLE_ID}</string>");
        assert!(
            plist.contains(&expected),
            "Info.plist bundle id != BUNDLE_ID"
        );
    }

    #[test]
    fn refuses_to_trash_when_not_in_our_bundle() {
        // Test binaries aren't inside EverFlask.app.
        assert!(trash_running_bundle(ID).is_err());
    }
}
