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
    menu | toolbar | sidebar)
        ;;
    *)
        printf 'usage: %s controls|contrast|views|views-contrast|texts|texts-contrast|buttons|buttons-contrast|menu|toolbar|sidebar\n' "$0" >&2
        exit 2
        ;;
esac

case "$scene" in
    contrast | views-contrast | texts-contrast | buttons-contrast)
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
if [ "$scene" = toolbar ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_toolbar -- --ignored
fi
if [ "$scene" = sidebar ]; then
    exec cargo test -p bk --test kit --features bk/shell appkit_sidebar -- --ignored
fi
export APPKIT_LAYOUT APPKIT_CONTENT APPKIT_TAG
case "$scene" in
    texts | texts-contrast)
        cargo test -p bk --test kit --features bk/shell appkit_texts -- --ignored
        exec cargo test -p bk --test kit --features bk/shell appkit_focus -- --ignored
        ;;
    buttons | buttons-contrast)
        cargo test -p bk --test kit --features bk/shell appkit_buttons::appkit_buttons -- --ignored --exact
        exec cargo test -p bk --test kit --features bk/shell appkit_buttons_pressed -- --ignored
        ;;
esac
cargo test -p bk --test kit --features bk/shell appkit_scene -- --ignored
exec cargo test -p bk --test kit --features bk/shell appkit_pressed -- --ignored
