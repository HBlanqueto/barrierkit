m=$1
cd "$(dirname "$0")"
echo "# dock"; python3 kern.py base-$m 1363 1160 1240 1200 1668 1375 20 40
echo "# apple"; python3 kern.py apple-$m 222 135 295 215 180 200 41 41
echo "# spot"; python3 kern.py spot-$m 820 720 880 800 700 800 41 41
echo "# cc"; python3 kern.py cc-$m 395 2420 2520 2470 2440 380 41 8
echo "# apps"; python3 kern.py apps-$m 805 600 800 700 600 787 41 41
echo "# popover"; python3 kern.py win-$m 220 880 1020 950 900 150 41 41
