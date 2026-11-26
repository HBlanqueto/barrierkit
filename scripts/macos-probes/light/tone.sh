# tone.sh prefix x y w h: raw output per gray level, and the red/blue pair raw and in sRGB, at a patch
cd /private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/light
p=$1; shift
t=""; sd=""
for g in 0 16 32 48 64 96 128 160 192 224 240 255; do
  r=$(./rd data/$p-g$g.png patch "$@"); t="$t $(echo $r | awk '{printf "%.0f", $2}')"; sd="$sd $(echo $r | awk '{printf "%.1f", $6}')"
done
echo "$p tone:$t   (std$sd)"
for c in red blue; do echo "$p $c raw $(./rd data/$p-$c.png patch "$@" | cut -d' ' -f1-3)  srgb $(./rd data/$p-$c.png srgb "$@" | cut -d' ' -f1-3)  bare-srgb $(./rd data/$p-$c.png srgb 1300 500 41 41 | cut -d' ' -f1-3)"; done
