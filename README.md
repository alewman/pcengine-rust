# pcengine-rust

A Rust **PC Engine / TurboGrafx-16** board transcribed from
[pcengine-python](https://github.com/alewman/pcengine-python) at `v0.2.0`
(`4c32f26`) and proven equivalent to it: every CPU bus cycle
(physical address, value, kind, master clock), every frame's picture at the
9-bit palette value, every mixed audio sample and the CPU's state, frame by
frame, on the whole HuCard library. pcengine-python is the specification
(readable, cited, judged against Mesen2, Beetle PCE, Mednafen and Covell's
photographs); this board adds nothing and tunes nothing. Its CPU is
[huc6280-rust](https://github.com/alewman/huc6280-rust) (crate `huc6280`,
transcribed from huc6280-python `v0.4.0`, the revision pcengine-python
`v0.2.0` pins). The add-ons
[supergrafx-rust](https://github.com/alewman/supergrafx-rust) and
[pcenginecd-rust](https://github.com/alewman/pcenginecd-rust) plug into its
socket traits, and
[pcengine-rust-fast](https://github.com/alewman/pcengine-rust-fast) is its
speed fork. MIT licensed. Built with Claude (Claude Fable 5.2, through Claude
Code) for alewman, and prepared for publication by Claude Opus 5.5. No ROMs,
System Card or other BIOS are included: bring your own.

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
traits, and save states.

## Status

**2026-11-13: 0.2.0, following pcengine-python `v0.2.0` (its issue 0004
settled) and huc6280-rust 0.2.0; all seven rungs pass against the new
reference.** Every rung compares this board with the reference, frame by
frame:

| Rung | What | Result |
| --- | --- | --- |
| 1 | Covell's CPU test runs 60 frames on the board | passed: master clock 21,538,338, a picture every frame |
| 2 | The lockstep gate on Covell's test and the synthetic probe card | identical |
| 3 | The 18 picture-gate titles (600 frames each, pad input) and the five PSG games (900) | 23 of 23 identical |
| 4 | The library: every HuCard, 300 frames from power-on with a RUN press | 180 of 180 identical (179 cards and the Street Fighter II' card) |
| 5 | A test device in every socket, the machine rows crossed, each named switch away from its default | 71 of 71 identical; 37 of 37 switches and devices change at least one case |
| 6 | Save states: round trip and run-ahead on the 18 titles | 18 of 18, twice each; run-ahead equal on all 18; a 0.1.0 state refused |
| 7 | Seeded faults | 56 caught, 0 missed (eight new, on what 0.2.0 carries) |

In all, **84,200 frames in 275 lockstep cases, and no case has ever
differed**: the transcription was identical to the reference on the first run
of each, at `v0.1.0` (2026-11-12) and again at `v0.2.0` (2026-11-13: the
reference cache rebuilt from the new reference, 275 of 275 identical on the
first run). The probe card's C18 section was made independent of its timing
for `v0.2.0` (the wait moved the probe's timeline and the held sample of
channel 2 happened to be 16, so `lfo_requires_ch2_on` changed nothing). **Nothing here was run on a real console**: this
board is proven equal to pcengine-python, and how close that is to the
hardware is the reference's record (its
[README](https://github.com/alewman/pcengine-python#readme) and `docs/`: M1
is closed as a documented provisional, the picture and library gates are
judged against emulators). Not covered: a window or player (the speed fork
has one), the reference's scripts that drive the referee emulators, and the
add-ons, which are their own crates.

## Using it

```toml
[dependencies]
pcengine-rust = { git = "https://github.com/alewman/pcengine-rust", rev = "<commit>" }
```

(Pin the commit you have checked; the crate is not on crates.io. It brings
huc6280-rust by the git revision in its own `Cargo.toml`.) Rust 1.85 or
newer. The library is `pcengine_rust`, and `pcengine_rust::huc6280`
re-exports the CPU core.

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

run(&[0u8; 0x2000]).unwrap(); // an empty 8 KiB image stands in for your card
```

This example is compiled and run by `cargo test` (as a doctest), as is the
one in `src/lib.rs`.

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
  standard conversion (the reference's `docs/vce.md`). Rows of one frame can
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
- **wasm**: nothing in the crate touches a file, a clock or a thread outside
  `trace::Setup::parse` and the binaries; `flate2` is the pure-Rust backend.
  The PSG's level curve is a written-out table of doubles (`LEVEL_BITS` in
  `psg/chip.rs`), not `powf` at run time, so every target has the same
  values and a state saved in a browser has a native build's bytes;
  `tests/psg_levels.rs` checks the table against the platform's `powf`.

### How an add-on crate plugs in

Every socket of the reference's
[docs/sockets.md](https://github.com/alewman/pcengine-python/blob/main/docs/sockets.md)
is a trait in `sockets.rs`, and `fake_devices.rs` has a device for each that
a real game runs on, in lockstep with the same device on the reference's
side:

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

### Layout

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
`tests/test_sockets.py`), `pads.rs`, `cards.rs`, `state.rs`,
`psg_levels.rs`, `vdc_wait.rs` (the reference's issue 0004 tests, and a
two-VDC decode made again after a wait).

## How it is proven

| What | Version |
| --- | --- |
| pcengine-python (the reference) | `v0.2.0`, `4c32f26730eb2a95041ec62547f977f97d2ab696` (0.1.0 was proven against `v0.1.0`, `bc9b976`) |
| huc6280-rust (`huc6280`) | `43e5978` (0.2.0, from huc6280-python `v0.4.0`, `265df88`, the revision the reference pins), by path until it is on GitHub; `Cargo.toml` |
| The HuCard library | 179 `.7z` images (the user's), and the Street Fighter II' card of the reference's `docs/cartridge.md` |
| Covell's CPU test | `CPU_Test.zip`, SHA-256 `db65a32d…6c937b625` (the reference's pin) |

Each rung reproduced with the script named, on Linux x86_64, rustc 1.93.1,
the reference on PyPy 7.3.20 (Python 3.11, `PYPY_GC_NURSERY=1M`):

| Rung | What | Result | Script |
| --- | --- | --- | --- |
| 1 | The skeleton: board, huc6280-rust and a plain ROM card run Covell's CPU test for 60 frames | passed: 60 frames, the master clock at 21,538,338, a picture every frame | `rung1.sh` |
| 2 | The lockstep gate, on Covell's CPU test: both boards, every frame's bus log, picture, audio and CPU state | identical, all 60 frames | `rung2.sh` |
| 3 | Picture and sound: the 18 picture-gate titles of the reference's `docs/m3-record.md`, 600 frames each with pad input, and the five PSG games of `docs/m4-record.md`, 900 frames | 23 of 23 identical: 10,800 frames of the 18 titles, 4,500 of the five | `rung3.sh` |
| 4 | The library: every HuCard, 300 frames from power-on with a RUN press, and the Street Fighter II' card; the smoke list of 20 first | smoke 20 of 20; then 180 of 180 identical (the 179 cards and Street Fighter II'): 54,000 frames | `rung4.sh` |
| 5 | Machines, sockets and switches: a test device in every socket, the machine rows crossed, a power-on seed, each named switch away from its default; and which case each one changes | 71 of 71 identical (14 socket cases, 3 machine cases, 53 switch cases, the probe's baseline); 37 of 37 switches and devices change at least one of their cases; `tests/sockets.rs` 17 tests, `tests/pads.rs` 12 | `rung5.sh` |
| 6 | Save states: the round trip and the run-ahead pass on the 18 titles; the refusals (a real 0.1.0 state among them), the wiring kept, and the synthetic-card round trip as unit tests | 18 of 18 titles equal, twice each; run-ahead equal on all 18; 10 unit tests | `rung6.sh` |
| 7 | Seeded faults: one single-line fault at a time in a copy of `src/`, run through the checks cheapest first until one fails | 56 caught, 0 missed: the 48 of 0.1.0 and eight on the VDC wait, the level table and the new CPU state field ([docs/seeded-faults.md](docs/seeded-faults.md)) | `rung7.sh` |

**The gate** (`scripts/lockstep.py`) runs each case of `scripts/cases.tsv`
on both boards: `scripts/pce_trace.py` on the reference's interpreter and
`target/release/pce-trace` here write one line a frame with the frame
counter, the master clock, the count and SHA-1 of the CPU bus log, the SHA-1
of the picture, the count and SHA-1 of the mixed audio and every field of the
CPU's state. A case passes when the two files are identical; otherwise the
first differing frame is named and narrowed to the first differing bus cycle.
The reference's traces are kept (`--reference-cache external/refcache`), so a
rerun costs the Rust side only: the whole gate of 275 cases then takes about
half a minute on 16 cores (about 25 minutes of 16 PyPy processes without the
cache). A switch that changes nothing proves nothing, so
`scripts/switch_effect.py` fails if a switch or device has no case it
changes; `scripts/make_probe.py` builds a synthetic 8 KiB card (this
repository's own code, not a ROM of anything) that reaches what the library's
first seconds do not. The cases, the probe, the socket devices and the save
state checks are described in [docs/ladder.md](docs/ladder.md).

### Reproducing

```text
scripts/fetch_pcenginepython.sh [DEST] [SOURCE]   # the reference at v0.2.0, with a PyPy venv
                                        # (CORE_SOURCE=a huc6280-python clone while its pin is not on GitHub)
scripts/fetch_covell.sh                 # Covell's CPU test by the reference's pin, into external/covell
scripts/make_cases.py                   # scripts/cases.tsv and smoke.txt from the library's names
scripts/rung1.sh [CPU_Test_10.pce]
scripts/rung2.sh
scripts/rung3.sh
scripts/rung4.sh --smoke                # the curated 20 first
scripts/rung4.sh                        # the library
scripts/rung5.sh
scripts/rung6.sh
scripts/rung7.sh                        # after a full gate run (it uses the reference cache)
python3 scripts/lockstep.py --reference-cache external/refcache --jobs 8   # the whole gate again
cargo test --release
```

What each needs: Rust 1.85 or newer; Python 3 for the scripts (standard
library only); PyPy for the reference (`PYTHON=/path/to/pypy3` for
`fetch_pcenginepython.sh`, default `pypy3` on the PATH); `7z` to read the
`.7z` cards; your own ROMs. The script defaults are the author's layout and
each is overridden by an environment variable: `LIB` (the directory of the
179 HuCard `.7z` images, named as No-Intro names them), `COVELL` (Covell's
`CPU_Test_10.pce`, which `fetch_covell.sh` fetches), `SF2` (the Street
Fighter II' card, which is not in the library) and `JOBS` (parallel cases, at
most 16). Archives are extracted one at a time into `scratch/` (gitignored)
and removed; the library is only read.

CI (`.github/workflows/ci.yml`, main and pull requests) runs `cargo fmt
--check`, `cargo clippy -D warnings`, `cargo test` (the sockets, the pads,
the mappers on synthetic images, the PSG table, the save-state round trip and
refusals on a synthetic card) and rung 1 on Covell's test, fetched by the
reference's pin. The lockstep gate needs the library and runs by hand.

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
  reference's order, `int()` as truncation. The PSG's level curve, which the
  reference computes with `10 ** x`, is the table of the doubles it computes
  (`LEVEL_BITS`), because `powf` is the platform's and under wasm32 nine of
  the 128 values came out one unit in the last place away.
- **`power_on_seed`** needs Python's `random.Random`: `mt.rs` is the Mersenne
  Twister with Python's integer seeding and `getrandbits`, so a seeded
  power-on gives the same RAM, VRAM, sprite table and colour table.
- **The VDC's wait** (`vdc/chip.rs`, `board.rs`, `sockets.rs`; the
  reference's issue 0004). The reference's VDC calls the board's
  `_serve_vram` from inside an access, and the board then runs every device,
  the pixel path included (the SuperGrafx's VPC, the device that made the
  access, mixes lines during the wait). A Rust device cannot be called again
  while it is in an access, so the access *stops* where the reference calls
  `serve` (`Vdc::serve` is a flag, `serve_wanted` the stop), the board serves
  the chip, holds the CPU for `served - mclk`, and makes the whole decode
  again. `VdcAccess` keeps the decode exact: the VDC accesses made before the
  stop are handed back from a log, the stopped one is made at the time the
  wait ended, and those after it (the lockstep pixel path writes both VDCs on
  `ST0`-`ST2`) are made after the wait with the device's own `mclk`, as the
  reference makes them. A device reaches a VDC's port with
  `BoardPins::vdc_read` / `vdc_write`.
- **Save states** (`state.rs`): the format is this crate's own (zlib of
  bincode behind a magic and a version), not the reference's tagged JSON of
  an object-graph walk; devices behind a socket save themselves
  (`DeviceState`). What holds is the reference's property and its refusals.
  The snapshot pair has no counterpart in the reference. The reference loads
  a 0.1.0 state into 0.2.0 with the CPU's new field as `None`; bincode names
  no field, so a state of format version 1 (pcengine-rust 0.1.0) is refused
  here with an error, not converted (`tests/state.rs` keeps a real one).
- **Not carried over**: the background's decode cache (a dict that only
  saves PyPy time), the PSG's step-stream listeners and the capture's
  `on_frame` callback (Python-side tooling), `from_path` constructors
  (`std::fs::read`), `Board.high_speed` (it is `pins.high_speed`).
- **Added for the gate**: `Board::bus_log` and `bus_detail` (the tap the
  reference's scripts make by wrapping the bus callables), `trace.rs`,
  `fake_devices.rs`.

Not transcribed: the reference's scripts that drive Mesen2, Beetle PCE, MAME
and Mednafen and the picture comparisons against them (the reference has
been judged against those; this board is judged against the reference), its
debugger-side `peek` users and its PNG writer.

## Speed

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

## Notes on the reference

Nothing found was ever a divergence: the Rust board did what pcengine-python
`v0.1.0` did in each case, and the gate ran through all of them. They were
places where the reference's behaviour looked unintended while it was being
transcribed, filed in pcengine-python as
[issue 0004](https://github.com/alewman/pcengine-python/blob/main/docs/issues/0004-findings-from-the-rust-port.md),
which is **settled** (2026-11-13, pcengine-python `v0.2.0`); this board
follows the settlement:

1. **`Psg.silent_units` was read once** (`_levels` was built in `__init__`,
   so setting it on an instance afterwards, as `scripts/psg_compare.py`
   applies `VARIANTS`, left the host-rate output as it was). **Fixed**: the
   table is rebuilt when the switch differs from the one it was built for
   (`Psg::level_table`, `levels_units`); `scripts/reference_notes.py` now
   shows the same level both ways.
2. **A CPU access that waited for a VRAM DMA ran the VDC ahead of the
   board** (`Vdc._data_access` caught the chip up to the transfer's end, lines
   past the board's line clock, so a waited-on transfer completed as if the
   display never started and `dma_resumes` could not matter for it). **Fixed**:
   the board runs every device, the line clock included, event by event until
   the access is served (`Board::serve_vram`), and holds the CPU to that time;
   `scripts/reference_notes.py` now prints 13,850 words left in both cases.
   How this is done without a callback from the VDC into the board that owns
   it is under "Where Rust forced a difference".
3. **`render_line`'s slice with a negative count**: left (unreachable); the
   reference's comment says why, and so does this board's.
4. **`Capture.finish_line`'s `self.path is None` branch**: left (reachable by
   design: the reference's `scripts/pixelpath_bench.py` sets it).
5. Small things: `Psg._acc_steps`, `Psg._reload_lfo_channels` and
   `PopulousCard.RAM_BANKS` are removed here too; the reference's
   `docs/sockets.md` now cites U13 for the video-bus pins; `Board.reset(POWER_ON)`
   is documented as not a power cycle (here too, on `Board::reset`).

The CPU core's own notes (interrupt edge cases, including one that made a
save state taken at a particular boundary continue differently) were
huc6280-python's issue 0005, settled in its `v0.4.0`; huc6280-rust 0.2.0
carries the settlement and this board carries it with the core (the CPU's
state now holds `polled_irq_lines`).

## Provenance

No GPL or LGPL source was copied, read or transcribed for this crate, and no
emulator's source was opened. The only sources transcribed are our own MIT
repositories, pcengine-python and (through huc6280-rust) huc6280-python, whose
trees are checked for copied or transcribed emulator code by pcengine-python's
`scripts/provenance_check.py`
([docs/provenance.md](https://github.com/alewman/pcengine-python/blob/main/docs/provenance.md)).
Emulators are named in the comments as the reference names them: as the
sources of a reading, not of code. One small table has a permissively
licensed origin, as in the reference: see [NOTICE.md](NOTICE.md).

[docs/handoff-brief.md](docs/handoff-brief.md) is the brief this board was
built from, kept as written, with what was built appended.

## Licence

MIT (see [LICENSE](LICENSE)), with the acknowledgement in
[NOTICE.md](NOTICE.md). Nothing third-party is committed: the reference, the
cards and Covell's test are fetched or read in place, and `.gitignore`
refuses the ROM, CD image and trace files.
