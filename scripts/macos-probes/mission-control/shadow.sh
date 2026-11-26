#!/bin/bash
B=${BIN:?}
O=${OUT:?}
name=$1; shift
export PROBE_DIR=$O
rm -f $O/ready $O/quit
$B/probe "$@" & pid=$!
i=0; while [ ! -f $O/ready ] && [ $i -lt 200 ]; do sleep 0.05; i=$((i + 1)); done
sleep 1
$B/ev m:20,1200
sleep 0.3
screencapture -x $O/$name-rest.png
osascript -e 'tell application "System Events" to key code 126 using control down'
sleep 1.3
screencapture -x $O/$name-mc.png
osascript -e 'tell application "System Events" to key code 53'
sleep 1
touch $O/quit
wait $pid
