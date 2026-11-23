#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
out=${APPKIT_OUT:-"$root/target/appkit"}
scene=${1:-}

case "$scene" in
    controls | contrast)
        APPKIT_LAYOUT=$out/layout-controls.tsv
        APPKIT_CONTENT=1600x1216
        APPKIT_TAG=
        ;;
    views | views-contrast)
        APPKIT_LAYOUT=$out/layout-views.tsv
        APPKIT_CONTENT=900x420
        APPKIT_TAG=views
        ;;
    texts | texts-contrast)
        APPKIT_LAYOUT=$out/layout-texts.tsv
        APPKIT_CONTENT=$(sed -n '1s/^# //p' "$APPKIT_LAYOUT")
        APPKIT_TAG=texts
        ;;
    buttons | buttons-contrast)
        APPKIT_LAYOUT=$out/layout-buttons.tsv
        APPKIT_CONTENT=$(sed -n '1s/^# //p' "$APPKIT_LAYOUT")
        APPKIT_TAG=buttons
        ;;
    swiftui | swiftui-contrast)
        APPKIT_LAYOUT=$out/layout-swiftui.tsv
        APPKIT_CONTENT=$(sed -n '1s/^# //p' "$APPKIT_LAYOUT")
        APPKIT_TAG=swiftui
        ;;
    lists | lists-contrast | containers | containers-contrast | scrollers | scrollers-contrast | tables | tables-contrast)
        family=${scene%-contrast}
        APPKIT_LAYOUT=$out/layout-$family.tsv
        APPKIT_CONTENT=$(sed -n '1s/^# //p' "$APPKIT_LAYOUT")
        APPKIT_TAG=$family
        ;;
    accents | accents-contrast)
        APPKIT_LAYOUT=$out/layout-accents.tsv
        APPKIT_CONTENT=$(sed -n '1s/^# //p' "$APPKIT_LAYOUT")
        ;;
    menu | toolbar | sidebar | picker | icons | lights | twin)
        ;;
    *)
        printf 'usage: %s controls|contrast|accents|twin|accents-contrast|views|views-contrast|texts|texts-contrast|lists|lists-contrast|containers|containers-contrast|scrollers|scrollers-contrast|tables|tables-contrast|buttons|buttons-contrast|swiftui|swiftui-contrast|menu|toolbar|sidebar|icons|lights\n' "$0" >&2
        exit 2
        ;;
esac

case "$scene" in
    contrast | accents-contrast | views-contrast | texts-contrast | lists-contrast | containers-contrast | scrollers-contrast | tables-contrast | buttons-contrast | swiftui-contrast)
        APPKIT_CONTRAST=1
        APPKIT_REDUCE=1
        APPKIT_TAG=${APPKIT_TAG:+$APPKIT_TAG-}hc
        export APPKIT_CONTRAST APPKIT_REDUCE
        ;;
esac

mkdir -p "$out"
export APPKIT_OUT="$out"
if [ "$scene" = menu ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_menu -- --ignored --nocapture
fi
if [ "$scene" = twin ]; then
    exec cargo test -p bk --test kit --features bk/shell swiftui_twin -- --nocapture
fi
if [ "$scene" = toolbar ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_toolbar -- --ignored
fi
if [ "$scene" = picker ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_picker -- --ignored
fi
if [ "$scene" = sidebar ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_sidebar -- --ignored
fi
if [ "$scene" = lights ]; then
    export APPKIT_BG_dark=262626 APPKIT_BG_hc_dark=282828
    cargo test -p bk --test kit --features bk/shell appkit_lights -- --ignored
    for tint in accent-blue:accent:007aff accent-orange:accent:ff9500 accent-purple:accent:af52de wallpaper-teal:wallpaper:1e8278 wallpaper-sand:wallpaper:e6d78c; do
        APPKIT_LIGHTS_SUFFIX=-${tint%%:*} APPKIT_LIGHTS_TINT=${tint#*:} \
            cargo test -p bk --test kit --features bk/shell appkit_lights -- --ignored
    done
    exit
fi
if [ "$scene" = icons ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_icon_lab -- --ignored
fi
export APPKIT_LAYOUT APPKIT_CONTENT APPKIT_TAG
case "$scene" in
    accents | accents-contrast)
        for accent in ${APPKIT_ACCENTS:-multicolor blue purple pink red orange yellow green graphite}; do
            if [ "$accent" = multicolor ]; then unset APPKIT_ACCENT; else export APPKIT_ACCENT="$accent"; fi
            export APPKIT_TAG="accents-$accent${APPKIT_CONTRAST:+-hc}"
            cargo test -p bk --test kit --features bk/shell appkit_scene -- --ignored
            cargo test -p bk --test kit --features bk/shell appkit_pressed -- --ignored
        done
        exit
        ;;
    texts | texts-contrast)
        cargo test -p bk --test kit --features bk/shell appkit_texts -- --ignored
        exec cargo test -p bk --test kit --features bk/shell appkit_focus -- --ignored
        ;;
    lists | lists-contrast | containers | containers-contrast | scrollers | scrollers-contrast | tables | tables-contrast)
        test=appkit_${scene%-contrast}
        light=ffffff
        dark=1e1e1e
        [ -z "${APPKIT_CONTRAST:-}" ] || dark=202020
        APPKIT_MODE=light APPKIT_BG=$light cargo test -p bk --test kit --features bk/shell "$test" -- --ignored
        APPKIT_MODE=dark APPKIT_BG=$dark cargo test -p bk --test kit --features bk/shell "$test" -- --ignored
        exit
        ;;
    buttons | buttons-contrast)
        cargo test -p bk --test kit --features bk/shell appkit_buttons::appkit_buttons -- --ignored --exact
        exec cargo test -p bk --test kit --features bk/shell appkit_buttons_pressed -- --ignored
        ;;
    swiftui | swiftui-contrast)
        exec cargo test -p bk --test kit --features bk/shell appkit_buttons::appkit_swiftui -- --ignored --exact
        ;;
esac
cargo test -p bk --test kit --features bk/shell appkit_scene -- --ignored
exec cargo test -p bk --test kit --features bk/shell appkit_pressed -- --ignored
