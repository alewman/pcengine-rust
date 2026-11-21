<!-- SPDX-License-Identifier: MIT -->
# Changelog

Not on crates.io; pin a commit (README, "Using it").

## Unreleased

### Pinned by git revision (2026-11-20)

- **huc6280-rust is a git pin again** (`e1d24a3`, on GitHub since 2026-11-20), not a path to a
  sibling clone; `Cargo.toml` names the `--config` patch that builds against a local checkout.
  CI no longer clones it beside the checkout.
- `scripts/fetch_pcenginepython.sh` with `CORE_SOURCE` installs the core with its dependencies:
  `--no-deps` had left out 6502-python, the core's frame. Its comments no longer say the
  reference is local only.

### The early display start (pcengine-python `03ac699`)

- **A display that starts before the row is cropped** (`vdc/chip.rs` `render_line`, transcribed
  from pcengine-python `03ac699`, local): a small HDS can put the display start inside the sync,
  so `first` is negative. The line is drawn into a work row padded on the left, the sprites draw
  into it (collision sees every display column), and the pad is dropped, so the row keeps
  1365 / divider dots. This crate indexed with `first as usize` and panicked
  (`src/vdc/chip.rs:353`); the reference's slice grew the row to 682 dots. Found by
  supergrafx-rust on its probe card in compat and base mode (its notes on the reference, item 7).
  New unit test `a_display_that_starts_before_the_row_is_cropped_and_the_row_keeps_its_length`.
- The gate's reference is pcengine-python `03ac699` (`scripts/fetch_pcenginepython.sh`, which now
  force-reinstalls the board too). The reference cache's key includes the reference's commit
  (`scripts/lockstep.py`), so a repin starts a fresh cache; the `3318027` cache was moved aside to
  `external/refcache-3318027`.
- `scripts/seeded_faults.py` copies the tree with fresh modification times (`copytree`'s `copy2`
  kept the old ones, so cargo could take a restored file for one older than the last fault's
  build; found by supergrafx-rust).

The gate against `03ac699`, the reference cache rebuilt from scratch: lockstep **275 of 275
identical** (the reference's traces byte-identical to `3318027`'s on all 275 cases: no case of
this gate reaches a display that starts before the row); rung 5 71 of 71 and 37 of 37 switches
and devices change a case; rung 6 18 of 18, run-ahead 18 of 18; rung 7 **63 caught, 3 missed**,
the same as before with the seeder fixed (no earlier catch was a stale build); `cargo test` all
pass; clippy `-D warnings` on rustc 1.93.1 and stable 1.99.1, fmt and the wasm32 build pass.

### The clean rewrite (pcengine-python `3318027`)

Follows pcengine-python's clean rewrite of 2026-11-19 (`3318027`, local, not pushed yet) and its
core's, huc6280-python `6ce348e`, through huc6280-rust `e1d24a3` (also local). The Rust code that
transcribed the withdrawn reference code was moved aside unread and transcribed again from the
new reference ([docs/restart.md](docs/restart.md)).

- **The VDC's line machine is new** (`vdc/sequencer.rs`, whole): line step at the end of the
  horizontal sync, display start (HDS + 1) characters later (the picture's origin moves 8 dots
  earlier: a 256-wide picture starts at dot 24 of the row, not 32), the scroll taken 16 dots before
  the display start (`SCROLL_LATCH_DOTS`, a closed-box fit upstream), the raster compare as the
  sync falls, vertical periods VSW + 1, VDS + 2, VDW + 1, VCR + 3 counted at line steps, burst mode
  at each VSW entry, a display cut by the vsync gets its vblank on the next line. New constants
  `CHARACTER_DOTS`, `FRONT_PORCH_EXTRA_LINES`, `RASTER_COMPARE_CLOCKS`, `SCROLL_LATCH_DOTS`; event
  enum `Event` (`Step`, `Latch`, `Display`, `Raster`, `Vblank`, `Collision`); `Sequencer::new(vblank_dots)`;
  `Sequencer::active_display` is a field (was a method).
- **The VDC chip** (`vdc/chip.rs`): a vertical scroll counter (`bg_y_counter`) loaded from BYR
  at the display period's start and on either BYR byte, +1 at every line step; `latch_scroll`;
  `line_begins`, `display_period_starts`, `vblank_begins` (VD, MWR bits 6-0 into `mwr_cache`, the
  SAT transfer); `fetch_sprites` at the line step with the manual's searched/fetched budget
  (2.4.1 (6)); `write` rewritten (VWR, MARR, LENR, DVSSR on the high byte; BYR on either byte), and
  it stops for the board's wait as `read` does. Removed: `latch_byr`, `latch_bxr`, `step_bg_y`,
  `line_counter_step`, `commit_unless_dma`; `vsync` no longer touches the scroll or MWR.
  `Vdc::new(power_on, vblank_dots, dma_resumes)` (the `vertical_at_line_start` argument is gone).
- **The DMAs** (`vdc/dma.rs`, whole): the SAT transfer runs at the vblank event and is busy for
  `SAT_CLOCKS` = 3 x 1365 master clocks (pcetech's "four scanlines"); it no longer sets burst mode
  at its end; a VRAM -> VRAM word is two accesses of `ACCESS_DOTS[VM]` dots each (VM as the last
  vblank took it); `dma_finish_without_display` replaces `dma_force_complete` for a VDC with no
  board. Removed `SAT_DOTS`, `DMA_DOTS_PER_WORD`.
- **Sprites**: `sprites::scan(vram, size, sat, line, mwr, limit, searched)`; `CLOCKS_PER_SPRITE`
  removed.
- **Board**: `BoardOptions::vertical_at_line_start` removed; `vblank_dots` default `Some(-16)`
  (was `Some(-8)`; the same place against Covell's photographs on the new geometry; -18 fits them
  best, a case of rung 5 now); `timer_underflow_window_mclk` default 3 (was 15, withdrawn).
- **Timer**: `underflow_window_mclk` 3 master clocks (one PPS3 clock, a hypothesis bounded by a
  closed-box measurement upstream).
- **Constants re-cited**: `HSYNC_CLOCKS`, `LINE_CLOCKS`, `VSYNC_LINES` from MAME huc6260.cpp's
  comments (BSD-3-Clause; NOTICE.md). Comments elsewhere no longer name GPL files or functions.
- **CPU core: huc6280-rust `e1d24a3`** by path from a sibling clone until it is pushed
  (`Cargo.toml` names the git pin to use then; CI clones it). `VanishedInterrupt::Latched` and
  `IrqPollCycle::Mednafen` are gone from the core (the board used neither).
- **Save states: format version 3.** The VDC's and the timer's layouts changed; versions 1 and 2
  are refused, not converted (`tests/data/state-v2.pcestate`, a real 0.2.0 state, is the new
  fixture).
- The gate: `--vertical-at-line-start` removed from `pce-trace` and the reference driver; the
  rung 5 cases `vertical-at-line-start*` replaced by `vblank-18` and `timer-window-15`; the
  reference fetch script takes the local clones and force-reinstalls the core. Seeded faults: the
  seven that held withdrawn lines cut unread, seventeen new ones on the new code; 63 caught, 3
  missed (the board's wait, missed in 0.2.0 too). The seeder copied no README into its tree, so
  `cargo test` failed there whatever the fault and counted it as caught (0.2.0's "56 caught, 0
  missed" was wrong for five); fixed, and it now checks the unfaulted board first. New tests: the
  DVSSR rule (only the high byte arms the SAT transfer), the version-2 refusal, the poll's lines
  in a state, MWR's SM taken at vblank, the waited write's time. Script defaults without machine
  paths (`LIB` required; `SF2` under `$NO_INTRO_DIR`; `COVELL` where `fetch_covell.sh` puts it).
  SPDX headers on every source, test and script file; the MAME BSD-3-Clause sources in NOTICE.md
  and the headers.

## 0.2.0 (2026-11-13)

Follows pcengine-python 0.2.0 (`4c32f26`) and its core, huc6280-python 0.4.0 (`265df88`).

- **CPU core: huc6280-rust 0.2.0** (`43e5978`, huc6280-python's issue 0005: the poll keeps the
  lines it saw). `CpuState` gained `polled_irq_lines: Option<u8>`; the gate's CPU text carries it.
  huc6280-rust 0.2.0 by its GitHub revision.
- **The board's own events run while the CPU waits on a VDC access** (the reference's issue 0004
  item 1): a VRAM DMA the CPU waits on now meets the display start, paused (C7, the default) or
  cancelled (`dma_resumes = false`, which serves the access there). `Board::step`, `settle` and
  `serve_vram` are the reference's `_step`, `_settle` and `_serve_vram`; `Vdc::serve` is the
  reference's `serve` wiring (a flag here), `data_access` returns the time the access is accepted
  at and the old loop is `wait_alone`. Because a Rust VDC cannot call the board that owns it, a
  waiting access stops and the board serves the chip and makes the decode again;
  `sockets::VdcAccess` hands back the VDC accesses made before the stop. Devices reach a VDC's port
  with `BoardPins::vdc_read` / `vdc_write` (new).
- **The PSG's level table follows `switches.silent_units`** set after construction (issue 0004
  item 2): `Psg::level_table`, `levels_units`.
- Removed with the reference: `Psg::acc_steps` (state), the no-op LFO reload, and
  `PopulousCard::RAM_BANKS`. `Board::reset` documents that a power-on reset is not a power cycle.
- **Save states: format version 2.** The CPU's state gained a field and the PSG's changed; a
  version 1 state is refused with an error, not converted (`state.rs`, `tests/state.rs` with a
  real 0.1.0 state).
- New tests: `tests/vdc_wait.rs` (the reference's two wait tests, the level table, and a two-VDC
  decode made again after a wait); seeded faults for each new behaviour.

## 0.1.0 (2026-11-12)

The board transcribed from pcengine-python v0.1.0 (`bc9b976`); all seven rungs pass.
