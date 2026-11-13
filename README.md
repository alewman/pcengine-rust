# pcengine-rust

A Rust PC Engine / TurboGrafx-16 board transcribed from
[pcengine-python](https://github.com/alewman/pcengine-python) at `v0.1.0`
and proven equivalent to it: every CPU bus cycle (physical address, value,
kind, master clock), every frame's picture at the 9-bit palette value, every
mixed audio sample and the CPU's state, frame by frame, on the whole HuCard
library. pcengine-python is the specification (readable, cited, judged
against Mesen2, Beetle PCE, Mednafen and Covell's photographs); this board
adds nothing and tunes nothing. It is the base for the speed fork and for
the add-on crates (`supergrafx-rust`, `pcenginecd-rust`), which plug into
its socket traits.

The board is the reference's, module for module, with the same names, every
named switch as a field with the reference's default, and the comments that
cite a source carried over: the machine table (PC Engine, TurboGrafx-16; a
row may ask for more VDCs and more RAM), the one timeline in master clocks
with the bus callbacks that price each cycle, the 8192-page decode, the
hardware page, the timer, the interrupt registers, the I/O port and its
buffer, the 2-button and 6-button pads and the multitap, the VCE, the frame
timeline and the line capture, the VDC (registers, bus port, the sequencer,
background, sprites, both DMAs), the six-channel PSG, the mixer, the cards
(ROM with mirroring, Street Fighter II', Populous, Tennokoe), the sockets as
traits, and save states. Where Rust forced a difference, the module header
says so, and they are listed below.

```text
cargo build --release
scripts/fetch_pcenginepython.sh         # the reference at v0.1.0 into external/, with a PyPy venv
scripts/rung1.sh                        # Covell's CPU test runs 60 frames
scripts/rung2.sh                        # ... in lockstep with the reference
```

The CPU is [huc6280-rust](../huc6280-rust) (crate `huc6280`, transcribed from
huc6280-python `v0.3.0`, the revision pcengine-python `v0.1.0` pins).
**`Cargo.toml` depends on it by path (`../huc6280-rust`) while it is local
only**: clone both side by side; the dependency is repinned to a GitHub
revision once both are pushed.

ROMs are bring-your-own, read in place and never committed (`.gitignore`
refuses their extensions). Nothing GPL or LGPL was read or transcribed for
this crate: the only source transcribed is pcengine-python, which is ours
and MIT. No emulator's source was opened.

## Certification

| What | Version |
| --- | --- |
| pcengine-python (the reference) | tag `v0.1.0`, `bc9b976f8b6ba29ff16fdcbd527978899d7e54b4` |
| huc6280-rust (`huc6280`) | `7f49bf8` (from huc6280-python `v0.3.0`, `bf20330`, the revision the reference pins), by path |
| The HuCard library | 179 `.7z` images (the user's), and the Street Fighter II' card of the reference's docs/cartridge.md |
| Covell's CPU test | `CPU_Test.zip`, SHA-256 `db65a32d…6c937b625` (the reference's pin) |

Ladder status at this commit, each rung reproduced with the script named, on
Linux x86_64, rustc 1.93.1, the reference on PyPy 7.3.20 (Python 3.11,
`PYPY_GC_NURSERY=1M`):

| Rung | What | Result | Script |
| --- | --- | --- | --- |
| 1 | The skeleton: board, huc6280-rust and a plain ROM card run Covell's CPU test for 60 frames | passed: 60 frames, the master clock at 21,538,338, a picture every frame | `rung1.sh` |
| 2 | The lockstep gate, on Covell's CPU test: both boards, every frame's bus log, picture, audio and CPU state | identical, all 60 frames | `rung2.sh` |
| 3 | Picture and sound: the 18 picture-gate titles of the reference's docs/m3-record.md, 600 frames each with pad input, and the five PSG games of docs/m4-record.md, 900 frames | 23 of 23 identical: 10,800 frames of the 18 titles, 4,500 of the five | `rung3.sh` |
| 4 | The library: every HuCard, 300 frames from power-on with a RUN press, and the Street Fighter II' card; the smoke list of 20 first | smoke 20 of 20; then 180 of 180 identical (the 179 cards and Street Fighter II'): 54,000 frames | `rung4.sh` |
| 5 | Machines, sockets and switches: a test device in every socket, the machine rows crossed, a power-on seed, each named switch away from its default; and which case each one changes | 71 of 71 identical (14 socket cases, 3 machine cases, 53 switch cases, the probe's baseline); 37 of 37 switches and devices change at least one of their cases; `tests/sockets.rs` 17 tests, `tests/pads.rs` 12 | `rung5.sh` |
| 6 | Save states: the round trip and the run-ahead pass on the 18 titles; the refusals and the synthetic-card round trip as unit tests | 18 of 18 titles equal, twice each; run-ahead equal on all 18; 8 unit tests | `rung6.sh` |
| 7 | Seeded faults | 48 caught, 0 missed (below) | `rung7.sh` |

**The gate** (`scripts/lockstep.py`) runs each case of `scripts/cases.tsv`
on both boards: `scripts/pce_trace.py` on the reference's interpreter and
`target/release/pce-trace` here write one line a frame with the frame
counter, the master clock, the count and SHA-1 of the CPU bus log (the tap of
the reference's own `scripts/state_roundtrip.py`: `address << 8 | value`,
`mclk << 2 | kind` for every cycle), the SHA-1 of the picture
(`capture.last_frame`: the rows at the 9-bit value, then the dividers), the
count and SHA-1 of the mixed audio (`board.take_audio()`) and every field of
the CPU's state. A case passes when the two files are identical. Otherwise
the first differing frame is named, and both boards are run again with
`--detail` on it, which narrows the difference to the first differing bus
cycle. The reference's traces are kept (`--reference-cache
external/refcache`, named by the case and a hash of its frames, image and
arguments), so a rerun costs the Rust side only: the whole gate of 275
cases then takes about half a minute on 16 cores (about 25 minutes of 16 PyPy processes without the cache). Archives are extracted one at a time
into `scratch/` and removed; the library is only read.

**In all, 84,200 frames in 275 cases, and no case has ever differed**: the
transcription was identical to the reference on the first run of each. (The
per-cycle narrowing has therefore only been exercised by the seeded faults.)

**Rungs 3 and 4.** US cards run on the TurboGrafx-16 row, the rest on the PC
Engine row. Rung 3's titles are the reference's picture gate: the first ten
of the library (Aero Blasters, After Burner II, Air Zonk, Alien Crush, Andre
Panza Kick Boxing, Armed F, Atomic Robo-Kid Special, Ballistix, Barbarian
for the Amiga 500 and for the Amstrad CPC), the four sprite-heavy games
(Blazing Lazers, R-Type, Bonk's Adventure, Samurai-Ghost) and the four
outliers (Barbarian for the ZX Spectrum, Death Sword for the PC, Mai Nurse,
Gunboat), each 600 frames with RUN at frame 250, I at 400 and right with II
from 480; and the PSG games (Aero Blasters, Alien Crush, Blazing Lazers,
Bonk's Adventure, Samurai-Ghost), 900 frames with two RUN presses and two
seconds of I and right. Rung 4's smoke list (`scripts/smoke.txt`) is the
nine cards the reference's docs/m5-record.md lists as different from Mesen2
(Santatlantean, Tiger Road, Bomberman, After Burner II, Davis Cup Tennis,
Galaga '90, Battle Royale, Super Metal Crusher, Night Creatures), the mapper
cards (Populous; Street Fighter II', also with a 6-button pad), the multitap
users (Bomberman '94 behind a multitap, Dungeon Explorer) and classics
(Blazing Lazers, Bonk's Adventure, R-Type, Gradius, Devil's Crush,
Splatterhouse). The Tennokoe card is not in the library; its mapper runs in
lockstep on the probe card built as a `TennokoeCard`, and in the unit tests.

**Rung 5.** The reference's own socket fakes (`tests/fake_devices.py`) record
calls and answer constants; they are transcribed (`src/fake_devices.rs`) and
drive the transcribed unit tests (`tests/sockets.rs`). For the lockstep,
three more devices are defined twice, line for line, in
`src/fake_devices.rs` and in `scripts/pce_trace.py`, and a game runs on each:

| Device | Sockets | What it does | Cases |
| --- | --- | --- | --- |
| `LockstepCard` | 1, 3, 4 | a ROM card that also claims banks `$F8-$FB`, so the work RAM is the card's, and that holds the CPU 3 master clocks on every 4096th write | Bonk's Adventure, the probe |
| `LockstepPort` | 2, 3, 5, 6, 9 | claims `$FF:1800-18FF`, is present (the I/O port's D7), raises IRQ2 by itself for 700 master clocks every 300,000, owns a 32 kHz audio source with a gain the mixer applies | Blazing Lazers, the probe |
| `LockstepPath` | 3, 7, 8, 12 | on a row with two VDCs: claims the VDC pages, sends `ST0`-`ST2` writes to both VDCs and other writes to the first (by `pins.opcode`), acknowledges both, lets the second VDC's pixel win where it is not zero | R-Type and the probe, with a power-on seed |
| `Pad6`, `MultiTap` (both readings of C10), no pad, a constant pad | 11 | the reference's own | Street Fighter II', Bomberman '94, the probe |

The machine rows are crossed (Blazing Lazers on the PC Engine row, where it
shows its region screen; Gradius and the probe on the TurboGrafx-16 row).
Each named switch is put away from its default on the probe card and on a
game: `power_on_seed`, `start_line`, `start_offset`, `vblank_dots` (`None`
and 0), `vertical_at_line_start` (with both vblank positions),
`dma_resumes`, `underflow_window_mclk` (0 and 1), `lines_follow_vce_bit2`,
the PSG's thirteen switches one at a time and its `mame` and `mednafen`
variants.

**A switch that changes nothing proves nothing**: if the reference's trace
with the switch is its trace without, both boards agree whether or not the
Rust board reads the switch. `scripts/switch_effect.py` compares each case's
reference trace with the same card's default trace and fails if a switch or
device has no case it changes. The library's first seconds do not reach
most of them (the LFO, DDA with the channel off, noise frequency 0, a DMA
running into the display period, the timer's reload window), so
`scripts/make_probe.py` builds a synthetic 8 KiB card, this repository's own
code, that does: 37 of 37 switches and devices change the probe's trace or a
game's.

**Rung 6.** As the reference's `scripts/state_roundtrip.py`: each of the 18
titles is saved 148 to 2,477 instructions into frame 300 (anywhere in a
frame, between two instructions, with pad input before and after), the state
loaded into a fresh system and into one that had run 150 frames, and the 60
frames after compared by the gate's own lines; the loaded system saved at
once gives the same bytes. States are 10 to 35 KiB. Then the snapshot pair:
120 frames, after each a snapshot, two frames run ahead with other buttons
held, the snapshot put back; every frame's line and the end state equal the
plain run's. A snapshot is 247 KiB (5 MHz titles) to 302 KiB (7 MHz).
`tests/state.rs` has the round trip on a synthetic card with a device in
every socket, the mappers' registers and RAM, and the refusals: bytes that
are not a state, another version, another machine row, another card (and
the same image behind another mapper, and an empty slot), a port device or
a pad the state has not, a second VDC; a refused state leaves the system
untouched.

### Seeded faults

`scripts/seeded_faults.py` plants one single-line fault at a time in a copy
of `src/`, builds it, and runs the checks cheapest first until one fails:
rung 2 (Covell's test and the probe card), rung 5 (the socket, machine and
switch cases), the smoke list of rung 4, `cargo test`, rung 3, the whole
library, rung 6.

| Fault | Module | What it changes | Caught by |
| --- | --- | --- | --- |
| `vce-access-not-stretched` | `board.rs` | only a VDC access takes the extra cycle, not a VCE one (C4, ares' reading) | rung 2 (`covell`) |
| `row-ends-a-dot-late` | `board.rs` | a line's picture row is handed to the capture 4 master clocks late | `cargo test` (`a_replacement_pixel_path_sees_every_line`) |
| `irq1-from-the-first-vdc-only` | `board.rs` | a second VDC's interrupt is not ORed into IRQ1 (the reference's issue 0001) | rung 5 (`sock-path-probe`) |
| `port-device-not-caught-up` | `board.rs` | the board never runs the port device up to its clock (the reference's issue 0002) | rung 5 (`sock-port-lazers`) |
| `disable-mask-not-delivered` | `board.rs` | a write to $1402 never reaches the CPU | rung 5 (`sock-multitap-bomberman94`) |
| `lines-delivered-only-on-advance` | `board.rs` | an interrupt line an access changed is seen by the CPU a cycle later | rung 2 (`base-probe`) |
| `timer-prescaler-not-reset` | `cpu_io/timer.rs` | the first tick after an enable comes a CPU cycle early (U24) | rung 2 (`base-probe`) |
| `timer-window-a-clock-long` | `cpu_io/timer.rs` | the count reads $7F one master clock longer after an underflow (C25) | rung 2 (`base-probe`) |
| `irq-status-buffer-bits` | `cpu_io/irq_ctl.rs` | $1402/$1403 return only four bits of the I/O buffer | rung 2 (`base-probe`) |
| `io-port-d4-low` | `cpu_io/io_port.rs` | the I/O port's D4 reads 0 | rung 2 (`covell`) |
| `read-does-not-load-the-buffer` | `cpu_io/mod.rs` | a read of the timer, the port or the interrupt registers leaves the I/O buffer | rung 2 (`base-probe`) |
| `pad6-counts-clr-level` | `pads.rs` | the 6-button pad changes scan on every write with CLR high, not on its rise | `cargo test` (`six_button_ignores_repeated_clr_high`) |
| `multitap-counts-under-clr` | `pads.rs` | the multitap's counter steps on a SEL rise while CLR is high | rung 5 (`sock-multitap-probe`) |
| `card-384k-upper-mirror` | `cards.rs` | a 384 KiB card's banks $40-$7F show the wrong part of the image | rung 5 (`sock-card-bonk`) |
| `tennokoe-lock-inverted` | `cards.rs` | the Tennokoe card's RAM unlocks on a zero | rung 5 (`sock-tennokoe-probe`) |
| `populous-ram-two-banks` | `cards.rs` | the Populous card's RAM takes writes in two banks of its four | rung 4, smoke (`lib-Populous (Japan) (En)`) |
| `palette-lsb-clears-bit-8` | `vce.rs` | a write of a colour's low byte clears its ninth bit | rung 2 (`covell`) |
| `palette-write-a-dot-late` | `video.rs` | a palette write made exactly at a dot's start shows from the next dot | rung 2 (`covell`) |
| `line-count-ignores-the-machine-row` | `video.rs` | `lines_follow_vce_bit2 = false` still follows VCE bit 2 (C3) | rung 5 (`sw-no-bit2-lines-probe`) |
| `cpu-slot-every-fourth-dot` | `vdc/chip.rs` | the CPU's VRAM slots repeat every 4 dots, not 8 (U1) | rung 5 (`sock-path-rtype`) |
| `raster-compare-a-line-early` | `vdc/chip.rs` | the raster compare fires a line early | rung 2 (`base-probe`) |
| `byr-set-on-the-msb-only` | `vdc/chip.rs` | a write of BYR's low byte alone does not move the line counter (Mednafen's note) | rung 2 (`base-probe`) |
| `byr-latched-a-dot-late` | `vdc/sequencer.rs` | BYR is latched a dot later in the line (U3) | rung 2 (`base-probe`) |
| `vds-a-line-short` | `vdc/sequencer.rs` | the display period starts a line early | rung 2 (`covell`) |
| `burst-from-the-background-bit` | `vdc/sequencer.rs` | burst mode is decided from the background enable alone | rung 5 (`mach-tg16-gradius`) |
| `sat-transfer-time-fixed` | `vdc/dma.rs` | the SAT transfer takes 1024 dots of the 5 MHz clock whatever the dot clock | rung 2 (`base-probe`) |
| `dma-cancelled-by-the-display` | `vdc/dma.rs` | a VRAM DMA running when the display starts is cancelled whatever `dma_resumes` says (C7) | rung 2 (`base-probe`) |
| `seventeen-sprites` | `vdc/sprites.rs` | a line holds 17 sprite entries before the overflow | rung 2 (`base-probe`) |
| `sprites-always-in-front` | `vdc/sprites.rs` | a sprite pixel hides the background whatever its priority bit | rung 2 (`base-probe`) |
| `bat-height-from-the-width-bit` | `vdc/background.rs` | the BAT's height follows MWR D5 | rung 2 (`base-probe`) |
| `vdw-eight-bits` | `vdc/regs.rs` | the display height register is 8 bits wide | rung 2 (`base-probe`) |
| `noise-frequency-not-complemented` | `psg/chip.rs` | the noise period is taken from NF, not its complement | rung 2 (`base-probe`) |
| `noise-feedback-bit` | `psg/chip.rs` | the noise shift register is 17 bits long | rung 2 (`base-probe`) |
| `lfo-shift` | `psg/chip.rs` | the LFO's modulation is shifted by 0, 1, 2 where a period is first loaded | rung 2 (`base-probe`) |
| `key-on-steps-the-address` | `psg/chip.rs` | key-on does what the other `keyon_mode` says (C22) | rung 2 (`base-probe`) |
| `dda-latch-needs-on` | `psg/chip.rs` | R6 in DDA mode loads the latch only with the channel on (C16, Mednafen's way) | rung 2 (`base-probe`) |
| `mixer-rounds-half-away` | `mixer.rs` | the mixer rounds a half away from zero (Rust's `round`), not to even (Python's) | rung 5 (`sock-port-lazers`) |
| `mixer-hold-rounds` | `mixer.rs` | a source at another rate is held by a rounded count | rung 5 (`sock-port-lazers`) |
| `mixer-gain-twice` | `mixer.rs` | a source that does not apply its own gain gets it doubled | rung 5 (`sock-port-lazers`) |
| `hsync-a-clock-short` | `machine.rs` | the horizontal sync is 236 master clocks | rung 2 (`covell`) |
| `irq-line-is-a-single-flag` | `sockets.rs` | a source letting go of an interrupt line lowers it for every source | `cargo test` (`irq_line_is_a_wired_or_of_sources_never_a_max`) |
| `work-ram-half-size` | `pagetable.rs` | work RAM mirrors every 4 KiB | rung 2 (`covell`) |
| `interrupt-registers-unmapped` | `hardware_page.rs` | $FF:1400-17FF answers $FF | rung 2 (`base-probe`) |
| `state-forgets-the-capture` | `state.rs` | a loaded state keeps the picture rows of the system it is loaded into | `cargo test` (`a_loaded_run_continues_bus_cycle_for_bus_cycle`) |
| `state-forgets-the-pad-latch` | `state.rs` | a loaded state keeps the SEL/CLR latch of the system it is loaded into | rung 6 (`pic-Aero Blasters (USA)`) |
| `state-takes-the-timer-switch` | `state.rs` | a loaded state overwrites the system's own `underflow_window_mclk` | `cargo test` (`loading_keeps_the_wiring_and_the_switches`) |
| `no-reset-requested` | `system.rs` | the CPU is put on the board without a reset: no start sequence, no reset vector | rung 2 (`covell`) |
| `seed-low-bits` | `mt.rs` | a seeded power-on takes the low bits of each random word | rung 5 (`sock-path-rtype`) |

**48 caught, 0 missed**: 28 by rung 2 (Covell's test or the probe card), 13
by rung 5, 1 by the smoke list, 5 by the unit tests (`tests/sockets.rs` and
`tests/pads.rs`, which rung 5 runs, and `tests/state.rs`, which rung 6
runs), 1 by rung 6's round trip on the titles. Rung 3 and the whole library
come late in the order and were never needed.

The first run of this list missed five. Three were holes in the cases, not
in the board: the probe card never wrote a scroll register on the dot it is
latched (`byr-latched-a-dot-late`), never set a display height past 8 bits
(`vdw-eight-bits`) and never keyed a channel on under the LFO (`lfo-shift`).
It does all three now. Two changed no behaviour, which makes them not
faults, and were dropped: the PSG's sample boundary tested with `>` where
the reference has `>=` (they differ only when a float sum lands exactly on
the tolerance), and `System::new` without its forced first delivery of the
interrupt lines (all three are low at power-on, which is what the CPU
already holds). `pad6-counts-clr-level` is caught by a unit test only: on
every pad scan the cases make, the faulty pad's count has the right parity.

### Speed

Speed is the fast fork's job; this is the figure to measure it against.
Frames per second from power-on, nothing logged (`pce-bench`,
`scripts/bench.py`), one core:

| Title (600 frames, as TurboGrafx-16, with pad input) | pcengine-rust | pcengine-python on PyPy 7.3.20 (whole run; second half) |
| --- | --- | --- |
| Blazing Lazers | 316-321 | 11.2; 11.5 |
| Bonk's Adventure | 296-331 | 11.8; 12.4 |
| R-Type | 305-337 | 8.6; 11.3 |

About 5.3 times real time (59.8 frames a second) against the reference's
0.19, some 27 times the reference, on a machine at load 30 (the Rust runs one
after another, three times each; the reference's three at once). The time is
the board's, not the CPU's: every CPU cycle runs the scheduler (`run_to`) and
asks each device for its next event, as the reference does.

## Where Rust forced a difference

Each module's header says so; together:

- **The CPU owns the board** (`system.rs`, `board.rs`). The reference's
  `System` holds a CPU and a board that hold each other. Here the board is
  the CPU's bus (`system.cpu.bus`, `system.board()`), and what the
  reference's board does to its CPU (`set_irq_lines`, `irq_disable`, reading
  `high_speed`) goes through the `Pins` each bus callback is handed, at the
  same points of the cycle and in the same order. `Board.attach_cpu` is
  `System::new`'s.
- **The decode is the board's** (`pagetable.rs`, `hardware_page.rs`). A
  page's handler is a name (`Handler::Ram`, `Vdc`, `Vce`, `CpuIo`, `Card`,
  `Port`, `PixelPath`), not an object; `Board::page_read` and `page_write`
  turn the name into the call.
- **`BoardPins` is handed to a device at every call** (`sockets.rs`), not
  once by `bind`: a Rust device cannot keep a reference into the board that
  owns it. It carries the IRQ2 line, `hold`, the VDC instances, the opcode
  being run and, for the port device and the pixel path, the card.
- **A device that is two objects in the reference is one trait object**:
  the pixel path may also claim pages (the reference's `Vpc` and
  `VideoPage`); a port device answers its own audio sources (the reference
  appends the CD unit's players to `board.audio_sources`), mixed after the
  PSG and before the board's list, the reference's order.
- **The VDC is one struct** (`vdc/`). `Sequencer` and `DmaEngine` are the
  state; their methods are `impl Vdc` blocks in `sequencer.rs` and `dma.rs`
  (`seq_*`, `dma_*`), where the reference has three objects calling each
  other. The `hold` callable is an accumulator the board takes when the
  access returns. Event kinds are an enum, not strings.
- **Switches.** Constructor arguments are `BoardOptions`; the PSG's class
  attributes are `PsgSwitches` (with `variant("mame")`, `variant("mednafen")`
  for `VARIANTS`); `Timer.underflow_window_mclk` and
  `MultiTap.clr_high_reads_pad` are fields; `keyon_mode` is an enum.
- **Python's arithmetic** is spelled out: `py_floordiv`, `round_ties_even`
  (`machine.rs`), the mixer's and the PSG's `f64` operations in the
  reference's order, `int()` as truncation.
- **`power_on_seed`** needs Python's `random.Random`: `mt.rs` is the Mersenne
  Twister with Python's integer seeding and `getrandbits`, so a seeded
  power-on gives the same RAM, VRAM, sprite table and colour table.
- **Save states** (`state.rs`): the format is this crate's own (zlib of
  bincode behind a magic and a version), not the reference's tagged JSON of
  an object-graph walk; devices behind a socket save themselves
  (`DeviceState`). What holds is the reference's property and its refusals.
  The snapshot pair has no counterpart in the reference.
- **Not carried over**: the background's decode cache (a dict that only
  saves PyPy time), the PSG's step-stream listeners and the capture's
  `on_frame` callback (Python-side tooling), `from_path` constructors
  (`std::fs::read`), `Board.high_speed` (it is `pins.high_speed`).
- **Added for the gate**: `Board::bus_log` and `bus_detail` (the tap the
  reference's scripts make by wrapping the bus callables), `trace.rs`,
  `fake_devices.rs`.

## Using the board (the fast fork, the browser page, an add-on)

```rust
use pcengine_rust::board::{Board, BoardOptions};
use pcengine_rust::cards::make_card;
use pcengine_rust::machine::TURBOGRAFX_16;
use pcengine_rust::pads::{self, Pad2};
use pcengine_rust::state::{load_snapshot, load_state, save_snapshot, save_state};
use pcengine_rust::system::System;

fn run(image: &[u8]) -> Result<(), String> {
    // A board from card bytes and a machine row (US cards want TURBOGRAFX_16).
    let board = Board::new(BoardOptions {
        machine: TURBOGRAFX_16,
        card: Some(make_card(image)?), // SF2 by size, Populous by its title, else plain ROM
        pad: Some(Box::new(Pad2::new())),
        ..BoardOptions::default()
    })?;
    let mut system = System::new(board);

    // One frame: hold buttons, run, take the sound and the picture.
    system.board_mut().set_pressed(0, pads::RUN | pads::I);
    system.run_frame();
    let _samples: Vec<(i16, i16)> = system.board_mut().take_audio(); // 44,100 Hz stereo
    if let Some(frame) = system.board().capture.last_frame.as_ref() {
        let table = &system.board().vce.rgb_table; // 512 entries of (r, g, b)
        for (row, _divider) in frame.rows.iter().zip(&frame.dividers) {
            // 242 rows (frame lines 14-255); a row has 1365 / divider dots: 341, 455 or 682
            let _rgb: Vec<(u8, u8, u8)> = row.iter().map(|&value| table[value as usize]).collect();
        }
    }

    // Save states (checked, compressed) and snapshots (neither; for run-ahead).
    let state: Vec<u8> = save_state(&system);
    load_state(&mut system, &state)?;
    let snapshot: Vec<u8> = save_snapshot(&system);
    load_snapshot(&mut system, &snapshot)
}
```

- **Machine rows**: `machine::PC_ENGINE`, `TURBOGRAFX_16`; `Machine { name,
  country_pc_engine, work_ram_bytes, vdc_count, lines_follow_vce_bit2 }` is a
  plain `Copy` struct an add-on makes its own row of (`Machine { work_ram_bytes:
  32 * 1024, vdc_count: 2, ..Machine::new("SuperGrafx", true) }`).
- **Cards**: `cards::make_card(&bytes) -> Result<Box<dyn Card>, String>`
  (strips a 512-byte copier header); `RomCard`, `Sf2Card`, `PopulousCard`,
  `TennokoeCard` (no detection: build it explicitly). An empty slot is `card:
  None`.
- **A frame**: `system.run_frame()` runs until the VCE starts the next frame;
  `system.step()` is one instruction; `system.frame()` the counter.
  `board.mclk` is the clock (21,477,272 Hz); a frame is 262 or 263 lines of
  1365.
- **The picture**: `board.capture.last_frame: Option<video::Frame>` with
  `number`, `rows: Vec<Vec<u16>>` and `dividers: Vec<u8>`. A value is the
  VCE's 9-bit colour `GGGRRRBBB` (not an index: the palette was looked up at
  each dot, so mid-line palette writes are already in the row).
  `vce.rgb_table[value]` is `(r, g, b)`; the default table is a linear 3-bit
  expansion (`vce::linear_rgb_table`), replaceable, because there is no
  standard conversion (the reference's docs/vce.md). Rows of one frame can
  differ in width (the dot clock may change between lines); dot 0 is the
  first dot after the horizontal sync, and a 256-wide picture starts at dot
  32 of 341.
- **Audio**: `board.take_audio() -> Vec<(i16, i16)>`: what every source has
  produced up to the clock now, 44.1 kHz stereo (about 737 pairs a frame).
  Call it once a frame. `board.psg.gain` scales the PSG.
- **Pads**: `BoardOptions::pad` takes any `ControllerPort`: `pads::Pad2`,
  `Pad6`, `MultiTap::new(vec![Some(pad), None, ...])` (five ports).
  `board.set_pressed(port, mask)` with the masks `pads::UP`, `RIGHT`,
  `DOWN`, `LEFT`, `I`, `II`, `SELECT`, `RUN`, `III`-`VI`; `port` is 0 for a
  pad plugged straight in and 0-4 behind a multitap.
- **Save states**: `state::save_state(&system) -> Vec<u8>` and
  `load_state(&mut system, &bytes) -> Result<(), String>` into a system built
  the same way (the same machine row, card, devices); another machine,
  another card (by the image's SHA-1) or a machine built another way is
  refused with the system untouched. `save_snapshot` / `load_snapshot` are
  the same body with no header, no check and no compression (about
  250-300 KiB, mostly the two pictures and the VRAM): take one, run ahead,
  put it back.
- **Switches**: every named switch of the reference is a field of
  `BoardOptions` (`power_on_seed`, `start_line`, `start_offset`,
  `vblank_dots`, `vertical_at_line_start`, `dma_resumes`, `psg`,
  `timer_underflow_window_mclk`), of `Machine` or of the device; the CPU's
  own are `system.cpu`'s public fields.
- **Reset**: `board.reset(ResetKind::PowerOn)` resets the card, the port
  device, the pixel path and the CPU's on-die peripherals, as the
  reference's; the CPU is `system.cpu.request_reset()`.

### How an add-on crate plugs in

Every socket of the reference's docs/sockets.md is a trait in `sockets.rs`,
and `fake_devices.rs` has a device for each that a real game runs on, in
lockstep with the same device on the reference's side:

| Socket | Trait | Plug it in | What it gets |
| --- | --- | --- | --- |
| 1 HuCard slot | `Card` | `BoardOptions::card` | `claims()` over the 8192 pages (not confined to the ROM window: a Super System Card's RAM at banks `$68-$7F`), `read`/`write` with the master clock, `grounds_cd`, `reset` |
| 2 Expansion port | `PortDevice` | `BoardOptions::port` | `claims()` (`$FF:1800-18FF`, RAM banks, ...), `present` (the I/O port's D7), `reset` |
| 3 One decode | `PageHandler` | a claim of a card, a port device or a pixel path | every bus access of the CPU, block transfers and `ST0-ST2` included |
| 4 Bus timing | `BoardPins::hold(clocks)` | inside any access or `catch_up` | the current CPU cycle stretched (RDY) |
| 5 Clock domains | `PortDevice::catch_up(mclk, pins)` | called by the board at every advance | time, without a register access |
| 6 Interrupts | `BoardPins::irq2: &mut IrqLine` | `pins.irq2.set("cd", level)` | a wired-OR level, one bit per source; IRQ1 is the OR of every VDC |
| 7 Pixel path | `PixelPath` | `BoardOptions::pixel_path` | per line, every VDC's row of 9-bit pixel values; returns the row for the VCE. It may also claim pages (the VPC decodes `$FF:0000-03FF`) and reach `pins.vdcs` |
| 8 Video timing | `VideoTiming` | the argument of `PixelPath::mix` | frame, line, the line's start, the dot divider, 262 or 263, in the vertical sync |
| 9 Audio | `AudioSource` | `PortDevice::audio_sources()`, or `board.audio_sources.push(...)` | `rate_hz`, `gain`, `take_samples(mclk)`; held to 44.1 kHz and summed |
| 10 Reset | `ResetKind` | `reset(kind, pins)` of each device | `PowerOn`, `Button` |
| 11 Controller port | `ControllerPort` | `BoardOptions::pad` | SEL and CLR, a nibble back; `set_pressed` from the host |
| 12 Instances | `Machine::vdc_count` | the machine row | `board.vdcs`: each run, synced, read for pixels and ORed into IRQ1 |

`BoardPins` (the argument of every device call) carries `irq2`, `hold`,
`vdcs` (the SuperGrafx's VPC steers each access of the VDC page to one of
them), `opcode` (the instruction being run: the VPC steers only
`ST0`-`ST2`) and `card` (for the port device and the pixel path: a CD unit
answers a Super System Card's id registers from the card, reached with
`Card::as_any_mut`). Every socket trait also carries `DeviceState`
(`save_state`, `load_state`, `state_identity`), which is how an add-on's
devices and media enter a save state. `sockets::DetachedPins` gives a
device's own tests a `BoardPins` with no board behind it.

For wasm: nothing in the crate touches a file, a clock or a thread outside
`trace::Setup::parse` and the binaries; `flate2` is the pure-Rust backend.
The PSG's level table is `10f64.powf(...)` at construction, which is the
platform's `pow`: a fork that must stay sample-identical across targets
should check that table against this crate's on the target (128 values).

## Layout

Each `src/*.rs` file is the transcription of the `pcengine_python` module of
the same name: `machine`, `sockets`, `pagetable`, `hardware_page`, `board`,
`system`, `cpu_io/` (`timer`, `irq_ctl`, `io_port`, `io_buffer`), `pads`,
`vce`, `video`, `vdc/` (`regs`, `chip`, `sequencer`, `background`,
`sprites`, `dma`), `psg/chip`, `mixer`, `cards`, `state`. Not from the
reference's package: `mt.rs` (Python's `random.Random`), `trace.rs` (the
gate's lines, the reference's press spelling, `roundtrip` and `runahead`),
`fake_devices.rs` (the reference's `tests/fake_devices.py`, and the three
lockstep devices). Binaries: `pce-trace`, `pce-state-roundtrip`,
`pce-bench`. Tests: `tests/sockets.rs` (the reference's
`tests/test_sockets.py`), `pads.rs`, `cards.rs`, `state.rs`.

Not transcribed: the reference's scripts that drive Mesen2, Beetle PCE, MAME
and Mednafen and the picture comparisons against them (the reference has
been judged against those; this board is judged against the reference), its
debugger-side `peek` users and its PNG writer.

## Notes on the reference

Nothing found is a divergence: the Rust board does what pcengine-python
`v0.1.0` does in each case, and the gate runs through all of them. They are
places where the reference's behaviour looked unintended while it was being
transcribed, left for its maintainers:

1. **`Psg.silent_units` is read once.** `Psg.__init__` builds `_levels` from
   it, so setting it on an instance afterwards (which is how
   `scripts/psg_compare.py` applies `VARIANTS`, with `setattr`) leaves the
   host-rate output as it was; `scripts/reference_notes.py` shows it (the
   level of 30 units stays 0.0, where a `Psg` built with 31 has 0.0056). The
   referee comparison itself is not affected: it works on the step stream and
   applies the curve itself. Here the switches are an argument of `Psg::new`.
2. **A CPU access that waits for a VRAM DMA runs the VDC ahead of the board.**
   `Vdc._data_access` calls `catch_up(due)` with the transfer's end, which can
   be many lines away, while the line clock (`hsync`, `vsync`) is the board's
   and arrives only after the access returns and the hold is applied. The
   transfer therefore always completes as if the display never started, and
   `dma_resumes` cannot matter for a transfer the CPU waits on. Found because
   the probe's first version read VRAM back right after starting a DMA and the
   switch then changed nothing.
3. **`render_line`'s slice assignment** (`row[first:end] = pixels[: end -
   first]`) would take a negative count and grow the row if the display
   started past the row's end; it cannot, because such a line's events are
   dropped by the next line's sync before it is drawn. Not reachable, noted
   because it reads as if it were handled.
4. **`Capture.finish_line`'s `self.path is None` branch** cannot be reached
   through the constructor, which replaces `None` with `DirectPath`.
5. Small things: `Psg._acc_steps`, `Psg._reload_lfo_channels` and
   `PopulousCard.RAM_BANKS` are never used; `sockets.py` cites U13 for the
   video-bus pins where docs/sockets.md says U15; `Board.reset(POWER_ON)`
   resets the CPU's on-die peripherals and the devices but not the VDC, the
   VCE, the PSG or the RAM.

## Reproducing

```text
scripts/fetch_pcenginepython.sh [DEST] [SOURCE]   # the reference at v0.1.0, with a PyPy venv
scripts/make_cases.py                   # scripts/cases.tsv and smoke.txt from the library's names
scripts/rung1.sh [CPU_Test_10.pce]
scripts/rung2.sh
scripts/rung3.sh
scripts/rung4.sh --smoke                # the curated 20 first
scripts/rung4.sh                        # the library
scripts/rung5.sh
scripts/rung6.sh
scripts/rung7.sh                        # after a full gate run (it uses the reference cache)
```

`$LIB` (the HuCard library, default `/data/emu/roms-retrobat/pcengine`),
`$COVELL` and `$SF2` name the user's files; `scripts/make_probe.py` builds
the synthetic probe card (this repository's own code, not a ROM of anything)
into `scratch/`. `JOBS` sets the parallel cases (at most 16). CI
(`.github/workflows/ci.yml`) runs `cargo fmt --check`, `cargo clippy -D
warnings`, `cargo test` and rung 1 on Covell's test, fetched by the
reference's pin; the lockstep gate needs the library and runs by hand.
`docs/handoff-brief.md` is the brief this was built from, with what was
built appended.

## Licence

MIT (see `LICENSE`). Nothing third-party is committed: the reference, the
cards and Covell's test are fetched or read in place.
