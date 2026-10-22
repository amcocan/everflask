#!/bin/sh
# Full local security check for the MacOS build. Run before every release.
# Builds the dev bundle and runs live tests against it; quits any running
# EverFlask first. Exits non-zero if any check fails.
set -u
cd "$(dirname "$0")/.."
APP=target/debug/EverFlask.app
BIN=$APP/Contents/MacOS/everflask
TMP=$(mktemp -d)
trap 'pkill -x everflask 2>/dev/null; rm -rf "$TMP"' EXIT
FAIL=0
pass() { echo "PASS  $1"; }
fail() { echo "FAIL  $1"; FAIL=1; }
check() { name=$1; shift; if "$@" >"$TMP/log" 2>&1; then pass "$name"; else fail "$name"; tail -5 "$TMP/log"; fi; }

echo "== Static"
check "rustfmt" cargo fmt --check
check "clippy -D warnings" cargo clippy --workspace --all-targets -- -D warnings
check "tests" cargo test --workspace
check "cargo audit (no vulnerabilities)" cargo audit
check "cargo deny (Mac targets)" cargo deny check

n=$(grep -rn "unsafe {" crates --include='*.rs' | wc -l | tr -d ' ')
s=$(grep -rn "// SAFETY:" crates --include='*.rs' | wc -l | tr -d ' ')
[ "$n" -le "$s" ] && pass "every unsafe block has a SAFETY comment ($n blocks)" || fail "unsafe blocks ($n) without SAFETY comments ($s)"
p=$(grep -rn "CGEvent::post" crates/*/src --include='*.rs' | wc -l | tr -d ' ')
[ "$p" -eq 1 ] && pass "exactly one input-posting call site" || fail "input-posting call sites: $p (expected 1)"
grep -rnE "TcpStream|UdpSocket|reqwest|hyper|ureq" crates/*/src >/dev/null && fail "network code in app" || pass "no network code in app"

echo "== Binary"
pkill -x everflask 2>/dev/null; sleep 1
check "build dev bundle" ./packaging/macos/dev-bundle.sh
codesign -dv "$APP" 2>&1 | grep -q "runtime" && pass "hardened runtime enabled" || fail "hardened runtime missing"
nm -u "$BIN" | grep -qE "^_(socket|connect|getaddrinfo)$" && fail "network symbols in binary" || pass "no network symbols in binary"
grep -qE "CFBundleURLTypes|NSAppTransportSecurity|NSServices" packaging/macos/Info.plist && fail "risky Info.plist keys" || pass "no URL schemes/services in Info.plist"

echo "== Live"
printf '#include <stdio.h>\n#include <stdlib.h>\n__attribute__((constructor)) static void p(void){FILE*f=fopen(getenv("PROBE_OUT"),"w");if(f){fputs("x",f);fclose(f);}}\n' > "$TMP/probe.c"
clang -dynamiclib -o "$TMP/probe.dylib" "$TMP/probe.c"
PROBE_OUT="$TMP/injected" DYLD_INSERT_LIBRARIES="$TMP/probe.dylib" "$BIN" >/dev/null 2>&1 & P=$!; sleep 2; kill $P 2>/dev/null; wait $P 2>/dev/null
[ -e "$TMP/injected" ] && fail "DYLD injection blocked" || pass "DYLD injection blocked"

pkill -x everflask 2>/dev/null; sleep 1
open "$APP"; sleep 2; open -n "$APP"; sleep 2; open -n "$APP"; sleep 2
[ "$(pgrep -x everflask | wc -l | tr -d ' ')" -eq 1 ] && pass "single instance (3 launches)" || fail "single instance (3 launches)"
pkill -x everflask; sleep 1
open -n "$APP" & open -n "$APP" & wait; sleep 3
[ "$(pgrep -x everflask | wc -l | tr -d ' ')" -eq 1 ] && pass "single instance (simultaneous launch)" || fail "single instance (simultaneous launch)"

# Hold real assertions the same way the app does, then SIGKILL.
cat > "$TMP/hold.py" <<'PY'
import ctypes, ctypes.util, time
k = ctypes.cdll.LoadLibrary(ctypes.util.find_library("IOKit"))
cf = ctypes.cdll.LoadLibrary(ctypes.util.find_library("CoreFoundation"))
cf.CFStringCreateWithCString.restype = ctypes.c_void_p
cf.CFStringCreateWithCString.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]
s = lambda b: cf.CFStringCreateWithCString(None, b, 0x08000100)
k.IOPMAssertionCreateWithName.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
for t in (b"PreventUserIdleDisplaySleep", b"PreventUserIdleSystemSleep"):
    k.IOPMAssertionCreateWithName(s(t), 255, s(b"everflask-crash-test"), ctypes.byref(ctypes.c_uint32()))
time.sleep(60)
PY
python3 -I "$TMP/hold.py" & H=$!; sleep 1.5; kill -9 $H; wait $H 2>/dev/null; sleep 1
pmset -g assertions | grep -q everflask-crash-test && fail "assertions released after SIGKILL" || pass "assertions released after SIGKILL"

echo
[ $FAIL -eq 0 ] && echo "All checks passed." || echo "Some checks FAILED."
exit $FAIL
