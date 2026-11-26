# pass.sh <d|l> [surfaces...]: series and kernel captures per surface, each opened, verified, closed and verified
. "$(dirname "$0")/lib.sh"
m=$1; shift
for s in "$@"; do
  case $s in apple) x=215;; spot) x=800;; cc) x=2470;; apps) x=700;; win) x=950;; base) x=1200;; esac
  [ $s != base ] && op $s
  echo "== $s open: $(wins | tr '\n' '|')"
  series $s-$m; kern $s-$m $x
  if [ $s != base ]; then cl $s; [ $s = apps ] && esc; fi
  left=$(wins)
  [ -n "$left" ] && { esc; left=$(wins); }
  echo "== $s closed, left: [$left]"
done
