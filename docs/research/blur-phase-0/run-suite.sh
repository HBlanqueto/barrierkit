. "$BLURP0/swegl.sh"
export MALAYSIA_DEV_SHELL=1
cd "$SCRATCH/malaysia"
for run in 1 2 3; do
  cargo test -p malaysia-compositor --message-format short -- --test-threads=4 --skip blur::within 2>&1 | grep -E "^test result: .* [0-9]+ passed|FAILED|panicked|skipping" | grep -v " 0 passed" | sort | uniq -c
done
