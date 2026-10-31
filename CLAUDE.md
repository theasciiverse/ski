# Ski! (Asciiverse packaging)

Mark Stevans' Ski! in Eric S. Raymond's Python port, packaged for the
Asciiverse catalog (id `ski`, kind `game`) as a one-file Rust port,
`src/main.rs`, like Terminal Rain. Not our game: keep the rules as they
are and list every change in PATCHES.md.

- `upstream` is gitlab.com/esr/ski (branch `master`); ours is branch
  `asciiverse`, `origin` github.com/theasciiverse/ski.
- The Python `ski` script stays untouched, as the reference.
- Release: tag `v<upstream version>-av.<n>` (first `v6.16-av.1`) and push
  it; `.github/workflows/release.yml` builds Linux musl, macOS and Windows.
- Line-based, not full-screen: prints one 75-column line per turn, reads a
  line at the `?` prompt. Needs 80 columns.
- Preview: `CASTREC=…/hub/tools/castrec.py asciiverse/record-preview.sh`
  (seed 164, fixed commands, about 48 s).
