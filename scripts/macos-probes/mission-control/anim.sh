#!/bin/bash
B=${BIN:?}
O=${OUT:?}
name=$1; shift
export PROBE_DIR=$O
rm -f $O/ready $O/quit
$B/probe "$@" & pid=$!
i=0; while [ ! -f $O/ready ] && [ $i -lt 200 ]; do sleep 0.05; i=$((i + 1)); done
sleep 1
$B/ev m:20,1250
sleep 0.3
$B/cap $O/anim-$name 0 0 2560 1440 640 360 10 k:126 s:1500 k:126 s:1500 k:126 s:1500 k:126 s:1500 k:126 s:1500 k:126 s:1200
touch $O/quit
wait $pid
