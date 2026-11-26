m=$1
cd /private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/light
echo "# dock top"; python3 edge.py base-$m col 1676 1400 1300
echo "# dock left"; python3 edge.py base-$m row 1395 760 680
echo "# apple bottom"; python3 edge.py apple-$m col 150 300 420
echo "# apple right"; python3 edge.py apple-$m row 222 280 400
echo "# spot bottom"; python3 edge.py spot-$m col 640 830 920
echo "# cc right of tile"; python3 edge.py cc-$m row 395 2500 2559
echo "# cc below last tile"; python3 edge.py cc-$m col 2300 500 640
echo "# apps right"; python3 edge.py apps-$m row 1000 1000 1120
echo "# apps top"; python3 edge.py apps-$m col 640 820 700
echo "# popover bottom"; python3 edge.py win-$m col 950 330 430
