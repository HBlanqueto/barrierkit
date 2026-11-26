#!/bin/sh
D=$(dirname "$0")/data
rm -f "$D/request"
open Probe.app --args "$1"
sleep 1
while pgrep -f Probe.app/Contents/MacOS/probe >/dev/null; do
  if [ -f "$D/request" ]; then screencapture -x -m "$(cat "$D/request")"; rm -f "$D/request"; fi
  sleep 0.01
done
