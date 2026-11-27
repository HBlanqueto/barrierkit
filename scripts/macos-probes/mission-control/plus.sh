#!/bin/bash
B=${BIN:?}
O=${OUT:?}
name=$1
secs=$2
cols=$3
shift 3
osascript -e 'tell application "System Events" to key code 53'
sleep 1
$B/ev m:1500,1000
$B/linecap $O/$name $secs 19,28,40,75,110,131 $cols k:126 "$@"
osascript -e 'tell application "System Events" to key code 53'
sleep 1
