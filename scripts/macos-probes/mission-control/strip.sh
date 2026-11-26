#!/bin/bash
B=${BIN:?}
O=${OUT:?}
M=$O/bdd
mode() { rm -f $M/ack; printf '%s' "$1" > $M/mode; i=0; while [ ! -f $M/ack ] && [ $i -lt 200 ]; do sleep 0.02; i=$((i + 1)); done; }
for bg in "$@"; do
  tag=$(echo "$bg" | tr ':,' '__')
  mode "$bg"
  $B/ev m:1500,1000
  osascript -e 'tell application "System Events" to key code 126 using control down'
  sleep 1.3
  screencapture -x $O/strip-c-$tag.png
  $B/ev m:600,20 s:100 m:602,21
  sleep 1.2
  screencapture -x $O/strip-x-$tag.png
  osascript -e 'tell application "System Events" to key code 53'
  sleep 1
  screencapture -x $O/strip-d-$tag.png
done
mode off
