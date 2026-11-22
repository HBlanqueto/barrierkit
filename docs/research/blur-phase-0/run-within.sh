. "$BLURP0/swegl.sh"
export MALAYSIA_DEV_SHELL=1 BLURP0_FRAMES=$SCRATCH/frames
cd "$SCRATCH/malaysia"
cargo test -p malaysia-compositor --lib --message-format short -- within --test-threads=1 --nocapture "$@" 2>&1
