#!/bin/bash
D=$(cd "$(dirname "$0")" && pwd)
while read -r name specs; do
  [ -n "$name" ] && "$D/layout.sh" "$name" $specs
done < "${1:-$D/arrangements.txt}"
