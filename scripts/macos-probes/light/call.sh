m=$1
cd "$(dirname "$0")"
c() { echo "# $1"; python3 corner.py ${W:-data/$2-$m-g255.png} data/$2-$m-g0.png $3 $4 $5 $6 $7 | tail -1; }
c dock-tl base 718 790 1350 1410 tl
c apple-bl apple 0 70 280 345 bl
c spot-bl spot 314 380 815 856 bl
c apps-tl apps 212 290 772 850 tl
c popover-tr win 990 1062 52 120 tr
