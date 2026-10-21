#!/bin/sh
# Removes EverFlask and everything it leaves behind. Use this if EverFlask
# was already deleted; otherwise prefer "Uninstall EverFlask..." in the menu.
# Mirrors crates/everflask-platform/src/macos/uninstall.rs; keep them in sync.
set -u
ID=io.github.amcocan.everflask

pkill -x everflask 2>/dev/null

# Permissions. tccutil only works while MacOS can still find the app, so
# this runs before the app is trashed. Never call it without an ID: that
# would reset the permission for every app on the Mac.
for svc in Accessibility PostEvent; do
    if /usr/bin/tccutil reset "$svc" "$ID" >/dev/null 2>&1; then
        echo "Reset $svc permission"
    else
        echo "Could not reset $svc permission (app not found). If EverFlask is"
        echo "  still listed in System Settings > Privacy & Security > Accessibility,"
        echo "  select it and click the minus button."
    fi
done

# Our entries in MacOS's "permission warning shown" list, and nothing else.
/usr/bin/defaults read com.apple.universalaccessAuthWarning 2>/dev/null |
    /usr/bin/sed -n 's/^ *"\{0,1\}\([0-9]*::[^"=]*\)"\{0,1\} = .*/\1/p' |
    /usr/bin/grep -iE "::($ID|.*/everflask\.app(/contents/macos/everflask)?)\$" |
    while IFS= read -r key; do
        /usr/bin/defaults delete com.apple.universalaccessAuthWarning "$key" &&
            echo "Removed warning record $key"
    done

# Preferences (via cfprefsd, then the file) and per-app folders.
/usr/bin/defaults delete "$ID" >/dev/null 2>&1
for p in \
    "$HOME/Library/Preferences/$ID.plist" \
    "$HOME/Library/Caches/$ID" \
    "$HOME/Library/HTTPStorages/$ID" \
    "$HOME/Library/Application Support/$ID" \
    "$HOME/Library/Saved Application State/$ID.savedState"; do
    if [ -e "$p" ] || [ -L "$p" ]; then
        /bin/rm -rf -- "$p" && echo "Removed $p"
    fi
done

# The app itself, if it's in a standard location. Moved to the Trash, not deleted.
for app in /Applications/EverFlask.app "$HOME/Applications/EverFlask.app"; do
    if [ -d "$app" ]; then
        /usr/bin/osascript -e "tell application \"Finder\" to delete POSIX file \"$app\"" >/dev/null &&
            echo "Moved $app to the Trash"
    fi
done

echo "Done."
