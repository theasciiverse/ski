#!/bin/sh
# Records the 100x30 catalog preview, asciiverse/ski.cast (hub/RECORDING.md),
# with hub's castrec.py: seed 164 and a fixed list of commands, typed about
# one a second. The skier launches an ICBM, summons the Fire Demon, who
# melts one Yeti, and is caught by the next. Then "n" to "Play again?".
#   CASTREC=path/to/hub/tools/castrec.py asciiverse/record-preview.sh
set -e
cd "$(dirname "$0")/.."
here=$(pwd)
castrec=${CASTREC:-$(cd ../../hub/tools && pwd)/castrec.py}
cargo build --release
# one entry per prompt; - is Enter alone
cmds="- - l - r - 3 j - l l - i - - r h - - d - 4 r - j - l - - t - - - r
      - - - - - - - - - - - - - - - - -"
script="w1.5"
for c in $cmds; do
  if [ "$c" != - ]; then script="$script;w0.5;s$c;w0.4;kenter"; else script="$script;w0.8;kenter"; fi
done
script="$script;w3;sn;w0.4;kenter;w1"
tmp=$(mktemp -d)
cp target/release/ski "$tmp/"
(cd "$tmp" && PATH="$tmp:$PATH" HOME="$tmp" \
  python3 "$castrec" "$here/asciiverse/ski.cast" . "$script" -- ski --seed 164)
rm -rf "$tmp"
