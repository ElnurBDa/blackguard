# Blackguard

A beautiful terminal implementation of **[Scoundrel](http://stfj.net/art/2011/Scoundrel.pdf)** — the
single-player rogue­like card game by Zach Gage and Kurt Bieg — built in Rust
with [ratatui](https://ratatui.rs).

The deck *is* the dungeon. Fight monsters, wield degrading weapons, sip potions,
and try to survive to the last card. (*Blackguard* is an old word for a
scoundrel — and a nod to the black-suit ♠♣ monsters you'll be fighting.)

```
        ♠♥ SCOUNDREL ♦♣                                    strict rules
   HP ♥♥♥♥♥♥♥♥♥♥♥♥♥♥······  14/20
  ╭ Room ─────────────────────────────────────╮╭ Equipment ──────────────╮
  │   ╔═════════╗   ╭─────────╮   ╭─────────╮  ││ Weapon  10♦             │
  │   ║K        ║   │7        │   │9        │  ││ hits monsters < 6       │
  │   ║    ♠    ║   │    ♥    │   │    ♦    │  ││ slain   8 6             │
  │   ║        K║   │        7│   │        9│  ││                         │
  │   ╚═════════╝   ╰─────────╯   ╰─────────╯  ││ Rooms cleared  0        │
  │     take -13      heal +7     weapon 9     ││ Deck           40       │
  ╰────────────────────────────────────────────╯╰─────────────────────────╯
```

## Install

```sh
# Rust users
cargo install blackguard

# Nix (no install — just run it)
nix run github:ElnurBDa/blackguard

# Or grab a prebuilt binary for your OS from the Releases page
# (static musl on Linux, macOS Intel/ARM, Windows .exe)
```

## Play

```sh
blackguard                 # random dungeon
blackguard --seed 42       # reproducible dungeon
blackguard --daily         # everyone's dungeon-of-the-day
blackguard --rule equal    # alternate weapon-binding rule
```

### Controls

| Key | Action |
| --- | --- |
| `←` / `→` (or `h` / `l`) | select a card |
| `Enter` / `Space` | use the selected card (smart: fight / equip / drink) |
| `f` | fight the selected monster **with your blade** |
| `b` | fight the selected monster **bare-handed** |
| `a` | avoid the room (send it to the bottom of the deck) |
| `n` / `r` | new game / retry the same seed |
| `t` | flip the weapon rule and replay the same deck |
| `?` | help · `q` quit |

## Rules in brief

- **44-card deck** — a standard deck minus the red face cards and red aces.
- **♠♣ monsters** cost you HP equal to their value (2–14).
- **♦ weapons** reduce a monster's damage by their value — but **degrade**: once
  a weapon slays a monster it can only be used on **weaker** monsters afterward.
- **♥ potions** heal by their value; only the **first each room** has any effect.
- Each room deals **4 cards**; play **3**, and the 4th carries into the next room.
- **Avoid** a room to bank it to the bottom of the deck — but never twice in a row.
- Clear the whole deck to **win** (score = remaining HP); drop to **0 HP** and you die
  (score = a negative tally of the monsters you never faced).

> **Weapon rule note.** The original rules are famously ambiguous about whether a
> bound weapon may re-kill a monster of *equal* value. Blackguard defaults to the
> stricter, community-standard reading (`--rule strict`) and lets you switch to
> `--rule equal` (or press `t` in-game).

## Build from source

No system Rust required — the toolchain comes from Nix:

```sh
nix develop            # or: nix-shell
cargo run
cargo test
cargo clippy --all-targets -- -D warnings
```

Architecture: a pure, fully unit-tested rules engine in [`src/core/`](src/core/)
(no I/O, deterministic from a seed) wrapped by a ratatui frontend
([`src/app.rs`](src/app.rs) update, [`src/ui/`](src/ui/) view,
[`src/tui.rs`](src/tui.rs) terminal loop).

## Acknowledgements

- **Scoundrel** was designed by **Zach Gage** and **Kurt Bieg** (2011) —
  [rules PDF](http://stfj.net/art/2011/Scoundrel.pdf). This is an independent
  fan implementation, not affiliated with the designers.
- Two other terminal implementations were great references while building this,
  and are well worth a look:
  - [`adrior11/scoundrel-tui`](https://github.com/adrior11/scoundrel-tui) — a Rust + ratatui version.
  - [`comradevanti-games/scoundrel_for_terminal`](https://github.com/comradevanti-games/scoundrel_for_terminal) — a Rust + termion version with a clean domain/UI split.

## License

MIT © 2026 — see [LICENSE](LICENSE).
