#!/bin/bash
B=${BIN:?}
O=${OUT:?}
name=$1; shift
export PROBE_DIR=$O
rm -f $O/ready $O/quit
$B/probe "$@" 2>$O/probe.err & pid=$!
i=0; while [ ! -f $O/ready ] && [ $i -lt 200 ]; do sleep 0.05; i=$((i + 1)); done
sleep 1
$B/ev m:20,1200
sleep 0.3
if $B/lw | awk '$2==0 && $1!="probe"' | grep -q .; then echo "foreign window on this Space"; touch $O/quit; wait $pid; exit 1; fi
screencapture -x $O/$name-rest.png
$B/lw | awk '$1=="probe" && $2==0' > $O/$name-frames.txt
pal=(255,0,0 0,255,0 0,0,255 255,255,0 0,255,255 255,0,255 255,128,0 128,0,255 0,128,64)
cols="${pal[*]:0:$#}"
echo "$cols" > $O/$name-cols.txt
osascript -e "tell application \"System Events\" to key code 126 using control down"
sleep 1.3
screencapture -x $O/$name-mc.png
osascript -e "tell application \"System Events\" to key code 53"
sleep 1
touch $O/quit
wait $pid
echo "== $name"
paste <(tail -r $O/$name-frames.txt | awk '{print $4,$5,$6,$7}') <($B/boxes $O/$name-mc.png 3 $cols)
