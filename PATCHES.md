# Changes from upstream

This is the Asciiverse packaging of Ski!, Mark Stevans' game (1983-1990) in
Eric S. Raymond's Python port (catb.org/~esr/ski, gitlab.com/esr/ski, the
`upstream` remote). Its history is kept whole, up to upstream's 6.16
(`a9fd185`, "Ready to ship 6.16."); everything after it is ours, on branch
`asciiverse`. BSD-2-Clause (`COPYING`), like the original.

## The Rust port (`src/main.rs`)

The `ski` script stays as it is; the catalog ships `ski`, the same game
ported to one Rust file, so it is one binary on Linux, macOS and Windows
and needs no Python or curses. The binary keeps upstream's command name.

Kept from the script, constant for constant: the 70-column slope and its
cellular-automaton terrain, the probabilities and how they grow each
level, the Yeti, ICBM and Fire Demon and how they move and melt, every
command (`l r j h t i d`, Enter, `!` for a shell command, `?` for help,
a count in front repeats a command, as `5l`), the help text, the terrain
key, every message and injury, and the score. It still prints one line
per turn and reads a line at the `?` prompt, as on a VT100.

Changed:

- **Play again.** After a run ends, it asks "Play again? (y/n)" instead of
  exiting at once, so the score stays on screen when av starts it (av
  redraws itself as soon as a title exits). `n`, end of input or Ctrl+C
  quits with "Bye!", as the script does on Ctrl+C.
- **Colours** go out as the plain ANSI codes 30 to 37, the same colours
  the script asked terminfo for. The player is drawn in reverse video:
  the script passed `curses.A_STANDOUT` to `setaf` as a colour number,
  which gave an unknown colour code, not the standout it meant. Colour is
  off when `NO_COLOR` is set or `TERM` is `dumb`; the script needed a
  terminfo entry and quit without one.
- **No crash at the right edge.** An ICBM chasing a Yeti near the right
  edge could overshoot it, and the script then stopped with an
  IndexError. Positions past either edge now wrap round, as Python's
  negative indexes already did at the left edge.
- **`--seed N`** makes a run repeatable (used for the catalog preview);
  `--version` prints the version.
