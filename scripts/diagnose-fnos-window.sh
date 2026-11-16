#!/bin/sh
# Motrix fnOS desktop window diagnostic (read-only, no elevated permissions)
set -u
app_root="${MOTRIX_APP_DEST:-/var/apps/motrix/target}"
sock="${app_root}/app.sock"
printf '%s\n' '== Motrix fnOS desktop diagnostic (read-only) =='
printf 'Target directory: %s\n' "$app_root"
if [ -d "$app_root" ]; then
  ls -ld "$app_root" 2>&1 || true
else
  echo 'Target directory not found'
fi
if [ -S "$sock" ]; then
  echo 'Unix Socket exists:'
  ls -l "$sock" 2>&1 || true
else
  echo 'Unix Socket MISSING (gateway cannot connect)'
fi
if ! command -v curl >/dev/null 2>&1; then
  echo 'curl not installed; HTTP checks skipped.'
  exit 0
fi
check() {
  description="$1"
  shift
  result=$(curl --max-time 7 -sS -o /dev/null -w '%{http_code}' "$@" 2>&1) || {
    printf '%s: connection error: %s\n' "$description" "$result"
    return
  }
  printf '%s: HTTP %s\n' "$description" "$result"
}
check 'Direct 17080 homepage' http://127.0.0.1:17080/
check 'Direct gateway-prefix homepage' http://127.0.0.1:17080/app/motrix/
if [ -S "$sock" ]; then
  check 'Unix Socket gateway homepage' --unix-socket "$sock" http://localhost/app/motrix/
  check 'Unix Socket asset prefix' --unix-socket "$sock" http://localhost/app/motrix/icon.png
fi
echo 'For the fnOS registration check, open /app/motrix/ under the exact URL used to access the fnOS desktop, while logged in.'
echo '404/not found: route registration. 502: Unix Socket. 200 HTML without JS/CSS: browser devtools Network/Console.'
