cd "$(dirname "$0")"
for m in d l; do
c() { echo "$m $1 $(python3 corner.py - data/$2-$m-g0.png $3 $4 $5 $6 $7 | tail -1)"; }
c apple-bl apple 0 19 290 345 bl
c spot-bl spot 314 344 815 856 bl
c apps-tl apps 212 245 772 830 tl
c popover-tr win 1030 1062 52 100 tr
echo "$m dock-tl pair $(python3 corner.py data/base-$m-g255.png data/base-$m-g0.png 718 790 1350 1410 tl | tail -1)"
done
