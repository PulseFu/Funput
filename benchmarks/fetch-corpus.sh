#!/bin/sh
# Fetch a large Vietnamese word list for the round-trip coverage check.
#
# We do NOT commit the corpus: its license differs from Funput's MIT. This script
# downloads it into ./.corpus/ (gitignored) so the benchmark is reproducible
# without vendoring third-party data. The committed `sample.txt` (our own,
# MIT-clean) is the default the benchmark runs against everywhere else.
#
# Source: Viet74K — github.com/duyet/vietnamese-wordlist (~74k entries).
set -eu

DIR="$(cd "$(dirname "$0")" && pwd)/.corpus"
URL="https://raw.githubusercontent.com/duyet/vietnamese-wordlist/master/Viet74K.txt"
OUT="$DIR/Viet74K.txt"

mkdir -p "$DIR"
echo "Downloading Viet74K → $OUT"
curl -fSL -o "$OUT" "$URL"
echo "Done. Run the headline benchmark with:"
echo "  cargo run -p funput-cli --release -- coverage benchmarks/.corpus/Viet74K.txt"
