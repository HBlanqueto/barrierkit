#!/bin/sh
# run.sh wallpaper mode tag [probe args...]
T=$(cd "$(dirname "$0")" && pwd); D=$T/data
wp=$1; shift
if [ "$wp" != "-" ]; then
  osascript -e "tell application \"System Events\" to tell every desktop to set picture to POSIX file \"$T/wp/$wp.png\"" >/dev/null
  sleep 2.5
  screencapture -x -m "$D/wp-$wp-$2.png"
fi
rm -f "$D/request"
open -n "$T/TintProbe.app" --args "$@"
sleep 1
while pgrep -f TintProbe.app/Contents/MacOS/probe >/dev/null; do
  if [ -f "$D/request" ]; then screencapture -x -m "$(cat "$D/request")"; rm -f "$D/request"; fi
  sleep 0.01
done
