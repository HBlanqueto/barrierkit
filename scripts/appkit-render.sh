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
    menu)
        ;;
    *)
        printf 'usage: %s controls|contrast|views|views-contrast|menu\n' "$0" >&2
        exit 2
        ;;
esac

case "$scene" in
    contrast | views-contrast)
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
export APPKIT_LAYOUT APPKIT_CONTENT APPKIT_TAG
cargo test -p bk --test kit --features bk/shell appkit_scene -- --ignored
exec cargo test -p bk --test kit --features bk/shell appkit_pressed -- --ignored
