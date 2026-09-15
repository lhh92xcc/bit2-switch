#!/bin/bash
set -euo pipefail
app=${1:?Usage: verify-macos-package.sh /path/to/bit2-switch.app}
plist="$app/Contents/Info.plist"
test -f "$plist"
name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleName' "$plist")
identifier=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$plist")
test "$name" = bit2-switch
test "$identifier" = ai.bit2.switch.desktop
executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist")
test -x "$app/Contents/MacOS/$executable"
test -f "$app/Contents/Resources/icon.icns"
test -f "$app/Contents/Resources/LICENSE"
test -f "$app/Contents/Resources/NOTICE.md"
/usr/bin/file "$app/Contents/MacOS/$executable"
/usr/bin/codesign --verify --deep --strict "$app"
printf '%s\n' 'Bundle identity, executable, icon, license, attribution, and code-signature integrity verified.'
