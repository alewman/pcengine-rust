//! The VDC's two DMAs and burst mode: VRAM -> VRAM, and VRAM -> sprite attribute table
//! (`vdc/dma.py`).
//!
//! [hw: pcetech 5.4; mfr: Hudson HuC6270 manual DCR/SOUR/DESR/LENR/DVSSR]
//!
//! VRAM -> VRAM: starts when LENR's MSB is written; one word per two dots (read, then write); SOUR
//! and DESR step up or down by DCR D2/D3 (not by the CR increment), LENR counts down and ends at
//! $FFFF; MAWR, MARR, the read buffer and the write latch are untouched. A transfer that is running
//! when the display period starts is **paused and resumes once the VDC is free again** (C7:
//! MacDonald, tier 2, and Mesen2 say so; Hudson's manual says the remainder is not transferred
//! and Mednafen cancels it; `resume = false` selects that reading). Time: 2 dots per word is
//! Mednafen's and Mesen2's.
//!
//! VRAM -> SAT: armed by a write to DVSSR, started when the display period ends (or every frame
//! with DCR D4); the 256 words are copied at the start and the VDC is busy for 1024 dots, which
//! is three lines at 5 MHz, "four lines after the last active line" as pcetech says (U2). The
//! done interrupt (DCR D0) follows.
//!
//! Burst mode (CR D6 and D7 clear when the frame starts) is held by the sequencer; the SAT
//! transfer and VRAM DMA are allowed whenever the display is not drawing.
//!
//! While either runs, CPU VRAM accesses wait (BSY).
//!
//! Differences forced by Rust: `DmaEngine` is the state; its methods are `impl Vdc` here, with
//! the reference's names behind `dma_`.

use serde::{Deserialize, Serialize};

use crate::vdc::chip::Vdc;
use crate::vdc::regs as r;
use crate::vdc::sequencer::VPhase;

/// [emu: Mednafen "1024 VDC cycles"; Mesen2 the same]
pub const SAT_DOTS: i64 = 1024;
pub const DMA_DOTS_PER_WORD: i64 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmaEngine {
    pub resume: bool,
    /// a VRAM -> VRAM transfer has been started and not finished
    pub running: bool,
    read_phase: bool,
    buffer: u16,
    /// master-clock time up to which the transfer has been run
    t0: i64,
    pub satb_pending: bool,
    /// the VDC is busy with the SAT transfer until then
    pub sat_end: Option<i64>,
}

impl DmaEngine {
    pub fn new(resume: bool) -> DmaEngine {
        DmaEngine {
            resume,
            running: false,
            read_phase: true,
            buffer: 0,
            t0: 0,
            satb_pending: false,
            sat_end: None,
        }
    }

    /// CPU VRAM accesses wait while a transfer runs, paused or not.
    #[inline]
    pub fn busy(&self) -> bool {
        self.running || self.sat_end.is_some()
    }
}

impl Vdc {
    /// The VRAM is free of display fetches (blanking, or burst mode).
    fn dma_allowed(&self) -> bool {
        self.dma.sat_end.is_none() && (self.seq.vphase != VPhase::Vdw || self.seq.burst_mode)
    }

    pub(crate) fn dma_next_event(&self) -> Option<i64> {
        let mut time = self.dma.sat_end;
        if self.dma.running && self.dma_allowed() {
            let mut halves = (self.regs.value[r::LENR] as i64 + 1) * DMA_DOTS_PER_WORD;
            if !self.dma.read_phase {
                halves -= 1; // the read of the current word is done
            }
            let at = self.dma.t0 + halves * self.seq.divider;
            time = Some(time.map_or(at, |t| t.min(at)));
        }
        time
    }

    // -- commands ---------------------------------------------------------------------------------

    /// LENR's MSB was written.
    pub(crate) fn dma_start(&mut self, now: i64) {
        self.dma_advance(now);
        self.dma.running = true;
        self.dma.read_phase = true;
        self.dma.t0 = now;
    }

    pub(crate) fn dma_arm_satb(&mut self) {
        self.dma.satb_pending = true;
    }

    pub(crate) fn dma_display_started(&mut self, now: i64) {
        self.dma_advance(now);
        if self.dma.running && !self.seq.burst_mode && !self.dma.resume {
            self.dma.running = false; // the manual's reading: the remainder is lost, no done IRQ
            self.commit_pending();
        }
        self.dma.t0 = now;
    }

    /// The SAT transfer test of Mednafen's Run loop: after the display period.
    pub(crate) fn dma_display_ended(&mut self, now: i64) {
        self.dma_advance(now);
        if self.dma.satb_pending || self.regs.value[r::DCR] & 0x10 != 0 {
            self.dma.satb_pending = false;
            let start = self.regs.value[r::SATB] as usize;
            for i in 0..256 {
                self.sat[i] = if start + i < self.vram.len() {
                    self.vram[start + i]
                } else {
                    0
                };
            }
            self.dma.sat_end = Some(now + SAT_DOTS * self.seq.divider);
        } else {
            self.seq.burst_mode = true;
        }
        self.dma.t0 = now;
    }

    // -- running ----------------------------------------------------------------------------------

    /// Run what is due up to master-clock time `t`.
    pub(crate) fn dma_advance(&mut self, t: i64) {
        if let Some(end) = self.dma.sat_end {
            if end <= t {
                self.dma.sat_end = None;
                if self.regs.value[r::DCR] & 0x01 != 0 {
                    self.status |= r::DS;
                }
                self.seq.burst_mode = true;
                self.commit_pending();
                self.dma.t0 = end;
            }
        }
        if !self.dma.running || !self.dma_allowed() {
            if self.dma.running {
                self.dma.t0 = t; // paused: no progress, the clock keeps moving
            }
            return;
        }
        let half = self.seq.divider; // one dot: the read, then the write, of each word
        while self.dma.running && self.dma.t0 + half <= t {
            if self.dma.read_phase {
                let src = self.regs.value[r::SOUR] as usize;
                self.dma.buffer = if src < self.vram.len() {
                    self.vram[src]
                } else {
                    0
                };
                self.dma.t0 += half;
                self.dma.read_phase = false;
                continue;
            }
            let dst = self.regs.value[r::DESR] as usize;
            if dst < self.vram.len() {
                self.vram[dst] = self.dma.buffer;
            }
            let dcr = self.regs.value[r::DCR];
            let regs = &mut self.regs.value;
            regs[r::SOUR] = if dcr & 0x04 != 0 {
                regs[r::SOUR].wrapping_sub(1)
            } else {
                regs[r::SOUR].wrapping_add(1)
            };
            regs[r::DESR] = if dcr & 0x08 != 0 {
                regs[r::DESR].wrapping_sub(1)
            } else {
                regs[r::DESR].wrapping_add(1)
            };
            regs[r::LENR] = regs[r::LENR].wrapping_sub(1);
            self.dma.t0 += half;
            self.dma.read_phase = true;
            if self.regs.value[r::LENR] == 0xFFFF {
                self.dma.running = false;
                self.commit_pending();
                if dcr & 0x02 != 0 {
                    self.status |= r::DV;
                }
            }
        }
    }

    /// A CPU VRAM access finds the VDC busy for an unknown time (paused DMA): finish it, as
    /// Mednafen does when nothing can wait for it (its 'DMA completion forced').
    pub(crate) fn dma_force_complete(&mut self, now: i64) {
        if let Some(end) = self.dma.sat_end {
            self.dma_advance(end);
        }
        let saved = self.seq.burst_mode;
        self.seq.burst_mode = true;
        self.dma.t0 = now - 2 * 65536 * self.seq.divider * DMA_DOTS_PER_WORD;
        self.dma_advance(now);
        self.seq.burst_mode = saved;
    }
}
