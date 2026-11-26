B=${BIN:?}
O=${OUT:?}
export PROBE_DIR=$O
SEQ=$(date +%s)
start() { local t=$1 exe=$2; shift 2; rm -f $O/ready-$t $O/cmd-$t;PROBE_TAG=$t PROBE_COLOR=${COLOR:-0} $B/$exe "$@" 2>$O/err-$t & local i=0; while [ ! -f $O/ready-$t ] && [ $i -lt 200 ]; do sleep 0.05; i=$((i + 1)); done; }
c() { local t=$1; shift; SEQ=$((SEQ + 1)); printf '%s %s' $SEQ "$*" > $O/cmd-$t.tmp; mv $O/cmd-$t.tmp $O/cmd-$t; sleep 0.1; }
sp() { $B/sp "$@"; }
mc() { osascript -e 'tell application "System Events" to key code 126 using control down'; sleep 1.3; $B/ev m:${2:-1280},${3:-700} s:100; sleep ${4:-0.2}; screencapture -x $O/$1.png; osascript -e 'tell application "System Events" to key code 53'; sleep 1; }
