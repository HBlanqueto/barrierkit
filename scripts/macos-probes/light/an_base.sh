# an_base.sh prefix: Dock patch, bare backdrop, menu bar clear test and ink, per gray and colour
cd "$(dirname "$0")"
p=$1
for s in g0 g16 g32 g48 g64 g96 g128 g160 g192 g224 g240 g255 red blue; do
  f=data/$p-$s.png
  echo "$s bare $(./rd $f patch 300 600 41 41 | cut -d' ' -f1-3) dock $(./rd $f patch 1668 1375 20 40) bar $(./rd $f cmp 4 26 32 600 2000) ink $(./rd $f minmax 20 4 460 22)"
done
