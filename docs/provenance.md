# Provenance

## Licence provenance (audit 2026-11-13)

pcengine-rust is a module-for-module transcription of pcengine-python (ours, MIT) and **inherits
pcengine-python's verdicts** (its `docs/provenance.md`, "Licence provenance (audit 2026-11-13)"),
audited from the records only, no GPL source opened. Carried over:

| Rust module | Python module | Verdict |
| --- | --- | --- |
| `src/vdc/sequencer.rs` | `vdc/sequencer.py`: Mednafen's `vdc.cpp` line machine "restated" | **rewrite** |
| `src/vdc/chip.rs` | `vdc/chip.py`: the scroll-latch state and `commit_unless_dma` follow Mednafen | **rewrite** (those parts) |
| `src/vdc/dma.rs` | `vdc/dma.py`: two Mednafen-placed functions | **unclear** |
| `src/machine.rs` | `HSYNC_CLOCKS = 237` with a quoted Mednafen line | **unclear** |
| `src/cpu_io/timer.rs` | `underflow_window_mclk = 15` read from Mesen2's code | **unclear** |
| `src/cards.rs` | MAME mappers (BSD-3) | **cite** |
| `src/psg/chip.rs` | MAME `c6280.cpp` volume table (BSD-3), notice in NOTICE.md | **cite** (done) |
| every other module (`board`, `vce`, `video`, `vdc/background`, `vdc/regs`, `vdc/sprites`, `cpu_io/*`, `pads`, `pagetable`, `hardware_page`, `system`, `sockets`, `state`, `mixer`, `fake_devices`) | the same modules | **clean** |

What the Rust tier added:

| Code | Source | Verdict |
| --- | --- | --- |
| `src/mt.rs` (Python's `random.Random` for seeded power-on) | MT19937 from Matsumoto and Nishimura's published description; CPython's behaviour as a closed box | **clean** |
| `src/trace.rs`, `src/bin/*`, `scripts/*` (lockstep, fault seeding, benches, `fetch_covell.sh` fetching Covell's test ROM by URL, never committed) | ours | **clean** |
| `tests/`, `tests/data/state-v1.pcestate` (a state of our own test program) | ours | **clean** |
| crates (Cargo.lock: serde, bincode, flate2, sha1 and their dependencies) | MIT/Apache-2.0/Zlib/Unicode-3.0, all permissive | **clean**; a binary distribution owes their notices |

Counts (rows above): clean 5, cite 2, rewrite 2, unclear 3.

provenance_check (2026-11-13, permissive trees only, run from pcengine-python) found nothing here.
History: 15 commits, no deleted files. The huc6280-rust core it depends on is audited separately.
