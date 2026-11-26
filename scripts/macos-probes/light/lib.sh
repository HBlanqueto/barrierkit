L=$(cd "$(dirname "$0")" && pwd)
D=$L/data
mode() { rm -f $L/ack; printf '%s' "$1" > $L/mode; i=0; while [ ! -f $L/ack ] && [ $i -lt 200 ]; do sleep 0.02; i=$((i+1)); done; }
shot() { mode "$2"; sleep 0.2; screencapture -x -m "$D/$1.png"; mode off; }
start_bd() { printf off > $L/mode; $L/bd >/dev/null 2>&1 & sleep 1; }
stop_bd() { printf quit > $L/mode; sleep 0.5; pkill -f "$L/bd" ; true; }
GRAYS="0 16 32 48 64 96 128 160 192 224 240 255"
series() { p=$1; for g in 0 16 32 48 64 96 128 160 192 224 240 255; do shot "$p-g$g" "gray:$g"; done; shot "$p-red" "rgb:208,74,34"; shot "$p-blue" "rgb:59,28,243"; }
kern() { p=$1; x=$2; shot "$p-e0255" "edgev:$x:0:255"; shot "$p-e2550" "edgev:$x:255:0"; shot "$p-e064" "edgev:$x:0:64"; shot "$p-lw" "line:$x:0:255"; shot "$p-lb" "line:$x:255:0"; }
esc() { osascript -e 'tell application "System Events" to key code 53'; sleep 0.8; }
op() { case $1 in
  apple) $L/ui press com.apple.finder 0 menubar >/dev/null;;
  spot) osascript -e 'tell application "System Events" to keystroke space using command down';;
  cc) $L/ui press com.apple.MenuBarAgent 3 >/dev/null;;
  nc) $L/ui press com.apple.MenuBarAgent 4 >/dev/null;;
  apps) open -a Apps;;
  win) mode win:on;;
  esac; sleep 1.2; }
cl() { if [ "$1" = win ]; then mode win:off; else esc; fi; }
wins() { $L/ui wins | awk '$2!=0' | grep -v "Window Server\|Wallpaper\|Fondo\|^Dock 20\|^Finder -2147483603"; }
finder() { m=$1; mode lvl:0; mode gray:128; osascript -e 'tell application "Finder" to activate' -e 'tell application "Finder" to make new Finder window' -e 'tell application "Finder" to set bounds of front Finder window to {1400, 300, 2200, 900}'; sleep 1.5; mode off
  for g in 0 64 128 192 255; do mode "gray:$g"; osascript -e 'tell application "Finder" to activate'; sleep 0.5; screencapture -x -m "$D/finder-$m-g$g.png"; mode off; done
  osascript -e 'tell application "Finder" to close front Finder window'; mode lvl:3; sleep 0.5; }
