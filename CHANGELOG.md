# Changelog

Not on crates.io; pin a commit (README, "Using it").

## 0.2.0 (2026-11-13)

Follows pcengine-python 0.2.0 (`4c32f26`) and its core, huc6280-python 0.4.0 (`265df88`).

- **CPU core: huc6280-rust 0.2.0** (`43e5978`, huc6280-python's issue 0005: the poll keeps the
  lines it saw). `CpuState` gained `polled_irq_lines: Option<u8>`; the gate's CPU text carries it.
  Until huc6280-rust 0.2.0 is on GitHub the dependency is by path (`../huc6280-rust`).
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
