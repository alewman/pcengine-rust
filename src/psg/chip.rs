//! The HuC6280's programmable sound generator: six wavetable channels, noise, LFO, DDA
//! (`psg/chip.py`).
//!
//! Written (in the reference) from Hudson's *HuC6280 PSG Manual* (tier 3: "manual" below) and
//! MacDonald's pcetech §10 and §2.4 (tier 2); where those are silent the choice is named in a
//! comment with a U/C number (docs/m4-record.md) and may be changed by a switch. There is no
//! hardware audio reference anywhere: **every output of this module is provisional, tier 5**.
//!
//! Time: one *step* is one PSG clock, master/6 = 3.58 MHz [pcetech 2.4; the manual's "7.16 MHz" and
//! its formulas' factor 2 agree]. A channel's waveform index advances every F steps (F = the 12-bit
//! frequency, 0 meaning 4096) [pcetech 10: $000 is the lowest frequency; manual 2.2.1:
//! f = fmaster / (2 x 32 x F)]. The model is event-driven: output is constant between index changes.
//!
//! Within a step every channel's output is taken from the state at the start of the step and the
//! counters then advance together (the manual says nothing of an order): ch 1's modulation of ch 0
//! therefore reads ch 2's sample *before* ch 2 advances in the same step.
//!
//! Differences forced by Rust: the class attributes are the fields of `PsgSwitches`
//! (`Psg::switches`), with `variant("mame")` and `variant("mednafen")` for the reference's
//! `VARIANTS`; `keyon_mode` is an enum. The step-stream listeners (the reference's referee
//! comparison, Python-side tooling) are not carried over. The host-rate arithmetic is the
//! reference's, in `f64`, operation for operation.

use serde::{Deserialize, Serialize};

use crate::cpu_io::PsgPins;
use crate::machine::{py_floordiv, round_ties_even};
use crate::sockets::{AudioSource, DeviceState};

pub const PSG_CLOCK_DIVIDER: i64 = 6; // master clocks per step [pcetech 2.4]
pub const NOISE_TAPS: [u32; 5] = [0, 1, 11, 12, 17]; // [emu: MAME and Mednafen agree; MacDonald's research] U6
pub const SILENT_UNITS: usize = 30; // [mfr 2.3] about 45 dB below the maximum "none is output"; 1 unit = 1.5 dB

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub frequency: i64,
    /// bit 7 ON, bit 6 DDA, bits 4-0 AL
    pub control: u8,
    pub balance: u8,
    pub wave: [u8; 32],
    pub index: usize,
    /// the DAC value held by DDA mode
    pub latch: u8,
    pub noise_control: u8,
    /// steps until the waveform index advances
    pub counter: i64,
    pub noise_counter: i64,
    pub lfsr: u32,
    /// C16: a sample forced by an R6 write until the next address step
    pub override_sample: Option<u8>,
}

impl Default for Channel {
    fn default() -> Self {
        Channel {
            frequency: 0,
            control: 0,
            balance: 0,
            wave: [0; 32],
            index: 0,
            latch: 0,
            noise_control: 0,
            counter: 1,
            noise_counter: 1,
            lfsr: 1,
            override_sample: None,
        }
    }
}

impl Channel {
    #[inline]
    pub fn on(&self) -> bool {
        self.control & 0x80 != 0
    }

    #[inline]
    pub fn dda(&self) -> bool {
        self.control & 0x40 != 0
    }
}

/// C22: what key-on (ON 0 -> 1) does to a waveform channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyOnMode {
    /// the counter restarts from F, the address stays (MAME)
    Reload,
    /// the counter resumes where it stopped and the address steps once (Mednafen)
    ResumeStep,
}

/// The points on which the manual and pcetech are silent or the two readable emulators differ
/// (docs/m4-record.md, C13-C23). Each default below follows the manual or pcetech where they
/// speak, else the larger share of the sources; `variant` flips them to one emulator's way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsgSwitches {
    /// C13: data written to R6 while a channel is ON and not DDA. pcetech: ignored, the index does
    /// not move. MAME and Mednafen store it in the waveform. True follows the two emulators.
    pub write_wave_while_playing: bool,
    /// C16: R6 in DDA mode loads the DAC latch: the manual says whenever DDA is set (also with the
    /// channel off); Mednafen only with the channel on, and it also lets an R6 write while playing
    /// override the sample until the next address step ("tested on a SuperGrafx").
    pub dda_latch_needs_on: bool,
    pub r6_overrides_sample_while_on: bool,
    /// C22: neither reading is in the manual.
    pub keyon_mode: KeyOnMode,
    /// C18: the LFO acts only when channel 2 is on (manual); MAME modulates whatever channel 2 is.
    pub lfo_requires_ch2_on: bool,
    /// C19: the LFO frequency register 0: 256 (Mednafen) or 1 (MAME's zero-length counter).
    pub lfo_zero_multiplier: i64,
    /// U6: the modulated frequency wraps in 12 bits (Mednafen) or not (MAME).
    pub lfo_mask12: bool,
    /// C23: LF TRG (R9 bit 7) halts channel 2 whatever LF CTL holds (Mednafen); MAME only with LF
    /// CTL non-zero.
    pub lf_trg_halts_always: bool,
    /// C15: R1/R5's table: the manual says 3 dB a step; both emulators share a table in which the
    /// step from 7 to 8 is 1.5 dB and from 8 to 9 4.5 dB (index 8 costs 15 units, not 14). The
    /// sixteen values in `scale` are that table, as in MAME's c6280.cpp (BSD-3-Clause, Charles
    /// MacDonald; see the reference's NOTICE.md).
    pub scale_table_quirk: bool,
    /// C14: attenuation at which a channel goes silent, in 1.5 dB units. The manual: 45 dB (30).
    /// Both emulators keep a 31st step (46.5 dB) that is still audible.
    pub silent_units: usize,
    /// False skips the host-rate output.
    pub render_audio: bool,
    /// C17: noise frequency NF' = 0 ("undefined" in the manual). MAME clocks the shift register
    /// every step; Mednafen every 32. 32 here.
    pub noise_zero_period: i64,
    /// C20: the noise shift register runs whether or not noise is selected (Mednafen); MAME runs it
    /// only while a channel is playing noise.
    pub noise_free_running: bool,
}

impl Default for PsgSwitches {
    fn default() -> Self {
        PsgSwitches {
            write_wave_while_playing: false,
            dda_latch_needs_on: false,
            r6_overrides_sample_while_on: false,
            keyon_mode: KeyOnMode::Reload,
            lfo_requires_ch2_on: true,
            lfo_zero_multiplier: 256,
            lfo_mask12: true,
            lf_trg_halts_always: true,
            scale_table_quirk: false,
            silent_units: SILENT_UNITS,
            render_audio: true,
            noise_zero_period: 32,
            noise_free_running: true,
        }
    }
}

impl PsgSwitches {
    /// Each readable emulator's way on the disputed points (the reference's `VARIANTS`):
    /// `"manual"` (the defaults), `"mame"`, `"mednafen"`.
    pub fn variant(name: &str) -> Option<PsgSwitches> {
        let manual = PsgSwitches::default();
        match name {
            "manual" => Some(manual),
            "mame" => Some(PsgSwitches {
                write_wave_while_playing: true,
                noise_zero_period: 1,
                noise_free_running: false,
                keyon_mode: KeyOnMode::Reload,
                lfo_requires_ch2_on: false,
                lfo_zero_multiplier: 1,
                lfo_mask12: false,
                lf_trg_halts_always: false,
                scale_table_quirk: true,
                silent_units: 31,
                ..manual
            }),
            "mednafen" => Some(PsgSwitches {
                write_wave_while_playing: true,
                dda_latch_needs_on: true,
                r6_overrides_sample_while_on: true,
                noise_zero_period: 32,
                noise_free_running: true,
                keyon_mode: KeyOnMode::ResumeStep,
                lfo_requires_ch2_on: true,
                lfo_zero_multiplier: 256,
                lfo_mask12: true,
                lf_trg_halts_always: true,
                scale_table_quirk: true,
                silent_units: 31,
                ..manual
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Psg {
    /// Switches, not state: a loaded state keeps the system's own.
    pub switches: PsgSwitches,
    pub channels: [Channel; 6],
    pub select: u8,
    /// R1
    pub balance: u8,
    /// R8
    pub lfo_frequency: u8,
    /// R9
    pub lfo_control: u8,
    /// steps run so far
    pub step: i64,
    // host-rate output (a box average of the step stream; outside the reference path)
    pub rate_hz: f64,
    pub gain: f64,
    /// up to 30 + 31 + 30
    levels: Vec<f64>,
    acc: [f64; 2],
    acc_steps: f64,
    phase: f64,
    out: Vec<(i32, i32)>,
    noise_period: [i64; 6],
}

impl Default for Psg {
    fn default() -> Self {
        Psg::new(44100.0, PsgSwitches::default())
    }
}

impl Psg {
    pub fn new(rate_hz: f64, switches: PsgSwitches) -> Psg {
        let levels = (0..128)
            .map(|u| {
                if u < switches.silent_units {
                    10f64.powf(-1.5 * u as f64 / 20.0)
                } else {
                    0.0
                }
            })
            .collect();
        let mut psg = Psg {
            switches,
            channels: Default::default(),
            select: 0,
            balance: 0,
            lfo_frequency: 0,
            lfo_control: 0,
            step: 0,
            rate_hz,
            gain: 1.0,
            levels,
            acc: [0.0, 0.0],
            acc_steps: 0.0,
            phase: 0.0,
            out: Vec::new(),
            noise_period: [0; 6],
        };
        for ch in 0..6 {
            psg.channels[ch].counter = psg.period(ch);
        }
        for ch in [4, 5] {
            psg.set_noise_period(ch);
            psg.channels[ch].noise_counter = 1;
        }
        psg
    }

    /// A loaded state keeps the system's own switches and the table made from them.
    pub(crate) fn keep_configuration(&mut self, live: &Psg) {
        self.switches = live.switches;
        self.levels = live.levels.clone();
    }

    // -- derived values -------------------------------------------------------------------------

    /// [mfr 2.1.10 note] the LFO acts only when channel 1 and 2 are both on; LF TRG halts it.
    fn lfo_modulating(&self) -> bool {
        if self.lfo_control & 3 == 0 || self.lfo_control & 0x80 != 0 {
            return false;
        }
        self.channels[1].on() || !self.switches.lfo_requires_ch2_on
    }

    fn mask12(&self, frequency: i64) -> i64 {
        if self.switches.lfo_mask12 {
            let masked = frequency & 0xFFF;
            if masked != 0 {
                masked
            } else {
                4096
            }
        } else {
            frequency.max(1)
        }
    }

    fn period(&self, ch: usize) -> i64 {
        let channel = &self.channels[ch];
        let mut frequency = if channel.frequency != 0 {
            channel.frequency
        } else {
            4096
        };
        if ch == 0 && self.lfo_modulating() {
            let shift = ((self.lfo_control & 3) as u32 - 1) * 2; // 0, 2, 4 [mfr 2.1.10]
            let modulation = self.channels[1].wave[self.channels[1].index] as i64 - 16; // [mfr table]
            frequency = self.mask12(channel.frequency + (modulation << shift));
        } else if ch == 1 && self.lfo_control & 3 != 0 {
            // [mfr 2.2.2] f_LFO = fmaster / (2x32xF2xFLF)
            frequency *= if self.lfo_frequency != 0 {
                self.lfo_frequency as i64
            } else {
                self.switches.lfo_zero_multiplier
            };
        }
        frequency
    }

    fn set_noise_period(&mut self, ch: usize) {
        let nf = 0x1F - (self.channels[ch].noise_control & 0x1F) as i64; // [mfr 2.3.3] NF is the complement
        self.noise_period[ch] = if nf != 0 {
            nf * 64
        } else {
            self.switches.noise_zero_period
        }; // 2 x 32 x 2 / 2 steps
    }

    /// Units of 1.5 dB below the maximum for a channel on one side (0 left, 1 right):
    /// R1 and R5 step 3 dB, R4 1.5 dB [mfr 2.1.2, 2.1.5, 2.1.6, 2.3]. 30 or more is silent.
    pub fn attenuation(&self, ch: usize, side: usize) -> usize {
        let channel = &self.channels[ch];
        let shift = if side == 0 { 4 } else { 0 };
        let main = (self.balance >> shift) & 0xF;
        let pan = (channel.balance >> shift) & 0xF;
        self.scale(main) + (31 - (channel.control & 0x1F) as usize) + self.scale(pan)
    }

    /// Units below the maximum for an R1/R5 nibble: 2 a step, 30 for 0 [mfr 2.1.2, 2.1.6].
    fn scale(&self, nibble: u8) -> usize {
        if self.switches.scale_table_quirk {
            const TABLE: [usize; 16] = [0, 3, 5, 7, 9, 11, 13, 15, 16, 19, 21, 23, 25, 27, 29, 31];
            return 31 - TABLE[nibble as usize];
        }
        2 * (15 - nibble as usize)
    }

    // -- the CPU's port --------------------------------------------------------------------------

    pub fn write(&mut self, offset: u32, value: u8, mclk: i64) {
        self.advance_to(py_floordiv(mclk, PSG_CLOCK_DIVIDER));
        self.write_register((offset & 0x0F) as u8, value);
    }

    fn write_register(&mut self, reg: u8, value: u8) {
        if reg == 0 {
            self.select = value & 7;
            return;
        }
        if reg == 1 {
            self.balance = value;
            return;
        }
        if reg == 8 {
            self.lfo_frequency = value;
            // counters pick up a changed period at their next reload
            return;
        }
        if reg == 9 {
            if value & 0x80 != 0 {
                // [mfr 2.1.10] LF TRG: ch 2's wave counter and address reset
                self.channels[1].index = 0;
                self.channels[1].counter = self.period(1);
            }
            self.lfo_control = value;
            return;
        }
        if reg > 7 || self.select > 5 {
            return; // "no more than 6 channels"
        }
        let select = self.select as usize;
        match reg {
            2 => {
                let channel = &mut self.channels[select];
                channel.frequency = (channel.frequency & 0xF00) | value as i64;
            }
            3 => {
                let channel = &mut self.channels[select];
                channel.frequency = (channel.frequency & 0x0FF) | (((value & 0x0F) as i64) << 8);
            }
            4 => self.write_control(select, value),
            5 => self.channels[select].balance = value,
            6 => self.write_wave(select, value & 0x1F),
            7 if select >= 4 => {
                self.channels[select].noise_control = value;
                self.set_noise_period(select);
            }
            _ => {}
        }
    }

    fn write_control(&mut self, ch: usize, value: u8) {
        let (was_dda, was_on) = (self.channels[ch].dda(), self.channels[ch].on());
        self.channels[ch].control = value;
        if value & 0x40 != 0 {
            self.channels[ch].index = 0; // [mfr 2.1.5] DDA: the address counter is reset
        } else if was_dda {
            self.channels[ch].index = 0; // [pcetech 10] reset when DDA goes 1 -> 0
            self.channels[ch].counter = self.period(ch);
        }
        if self.channels[ch].on() && !was_on && !self.channels[ch].dda() {
            if self.switches.keyon_mode == KeyOnMode::Reload {
                self.channels[ch].counter = self.period(ch);
            } else {
                self.channels[ch].index = (self.channels[ch].index + 1) & 31;
            }
        }
        self.channels[ch].override_sample = None;
    }

    fn write_wave(&mut self, ch: usize, value: u8) {
        let switches = self.switches;
        let channel = &mut self.channels[ch];
        if channel.dda() {
            if channel.on() || !switches.dda_latch_needs_on {
                channel.latch = value; // [mfr 2.1.7 (3)] "data is transferred directly to the D/A"
            }
        } else if !channel.on() {
            channel.wave[channel.index] = value; // [mfr 2.1.5] write data mode
            channel.index = (channel.index + 1) & 31;
        } else if switches.write_wave_while_playing {
            channel.wave[channel.index] = value;
        }
        if switches.r6_overrides_sample_while_on && channel.on() {
            channel.override_sample = Some(value);
        }
    }

    // -- running ----------------------------------------------------------------------------------

    /// Per channel (sample, units left, units right) for the step about to run; sample is
    /// -1 for a channel that is off (it contributes nothing).
    pub fn state(&self) -> [(i32, usize, usize); 6] {
        let mut out = [(0, 0, 0); 6];
        for (ch, channel) in self.channels.iter().enumerate() {
            let sample = if !channel.on() {
                -1
            } else if ch >= 4 && channel.noise_control & 0x80 != 0 {
                // [mfr 2.1.8] noise overrides the wave
                if channel.lfsr & 1 != 0 {
                    31
                } else {
                    0
                }
            } else if channel.dda() {
                channel.latch as i32
            } else if let Some(sample) = channel.override_sample {
                sample as i32
            } else {
                channel.wave[channel.index] as i32
            };
            out[ch] = (sample, self.attenuation(ch, 0), self.attenuation(ch, 1));
        }
        out
    }

    /// Run the steps up to (not including) step `target`.
    pub fn advance_to(&mut self, target: i64) {
        while self.step < target {
            let remaining = target - self.step;
            let mut n = remaining;
            let lfo = self.lfo_modulating();
            for (ch, channel) in self.channels.iter().enumerate() {
                if self.clocked(ch, channel) {
                    n = n.min(channel.counter);
                }
                if ch >= 4 && (self.switches.noise_free_running || plays_noise(channel)) {
                    n = n.min(channel.noise_counter);
                }
            }
            if n < 1 {
                n = 1;
            }
            let snapshot = self.state();
            self.emit(n, &snapshot);
            self.step += n;
            self.tick(n, lfo);
        }
    }

    /// [mfr 2.1.5] the waveform address advances only in mixing mode (ON, not DDA); LF TRG
    /// halts channel 2's counter (Mednafen: whatever the LF CTL bits).
    fn clocked(&self, ch: usize, channel: &Channel) -> bool {
        if !channel.on() || channel.dda() {
            return false;
        }
        if ch == 1 && self.lfo_control & 0x80 != 0 {
            return !self.switches.lf_trg_halts_always && self.lfo_control & 3 == 0;
        }
        true
    }

    fn tick(&mut self, n: i64, lfo: bool) {
        // the modulating sample is read before channel 2 moves in the same step
        let modulation = self.channels[1].wave[self.channels[1].index] as i64;
        for ch in 0..6 {
            if self.clocked(ch, &self.channels[ch]) {
                self.channels[ch].counter -= n;
                if self.channels[ch].counter <= 0 {
                    self.channels[ch].index = (self.channels[ch].index + 1) & 31;
                    self.channels[ch].override_sample = None;
                    if ch == 0 && lfo {
                        let shift = ((self.lfo_control & 3) as u32 - 1) * 2;
                        let frequency = self.channels[ch].frequency + ((modulation - 16) << shift);
                        self.channels[ch].counter = self.mask12(frequency);
                    } else {
                        self.channels[ch].counter = self.period(ch);
                    }
                }
            }
            if ch >= 4 && (self.switches.noise_free_running || plays_noise(&self.channels[ch])) {
                let period = self.noise_period[ch];
                let channel = &mut self.channels[ch];
                channel.noise_counter -= n;
                if channel.noise_counter <= 0 {
                    let mut bit = 0;
                    for tap in NOISE_TAPS {
                        bit ^= (channel.lfsr >> tap) & 1;
                    }
                    channel.lfsr = (channel.lfsr >> 1) | (bit << 17);
                    channel.noise_counter = period;
                }
            }
        }
    }

    // -- host-rate output ---------------------------------------------------------------------------

    fn emit(&mut self, n: i64, snapshot: &[(i32, usize, usize); 6]) {
        if !self.switches.render_audio {
            return;
        }
        let (mut left, mut right) = (0.0f64, 0.0f64);
        for &(sample, ul, ur) in snapshot {
            if sample >= 0 {
                left += self.levels[ul] * (sample - 16) as f64;
                right += self.levels[ur] * (sample - 16) as f64;
            }
        }
        let scale = self.gain / (6 * 16) as f64;
        let (left, right) = (left * scale, right * scale);
        let per_sample = 3_579_545.0 / self.rate_hz;
        let mut remaining = n as f64;
        while remaining > 0.0 {
            let room = per_sample - self.phase;
            let take = room.min(remaining);
            self.acc[0] += left * take;
            self.acc[1] += right * take;
            self.phase += take;
            remaining -= take;
            if self.phase >= per_sample - 1e-9 {
                self.out.push((
                    round_ties_even(self.acc[0] / per_sample * 32767.0) as i32,
                    round_ties_even(self.acc[1] / per_sample * 32767.0) as i32,
                ));
                self.acc = [0.0, 0.0];
                self.phase = 0.0;
            }
        }
    }

    /// Stereo samples at `rate_hz` up to `mclk` since the last call (a box average of the
    /// step stream; the reference check is the unresampled step stream).
    pub fn take_samples(&mut self, mclk: i64) -> Vec<(i32, i32)> {
        self.advance_to(py_floordiv(mclk, PSG_CLOCK_DIVIDER));
        std::mem::take(&mut self.out)
    }
}

fn plays_noise(channel: &Channel) -> bool {
    channel.on() && channel.noise_control & 0x80 != 0
}

impl PsgPins for Psg {
    fn write(&mut self, offset: u32, value: u8, mclk: i64) {
        Psg::write(self, offset, value, mclk);
    }
}

impl DeviceState for Psg {}

impl AudioSource for Psg {
    fn rate_hz(&self) -> f64 {
        self.rate_hz
    }

    fn gain(&self) -> f64 {
        self.gain
    }

    fn applies_gain(&self) -> bool {
        true // the mixer (mixer.rs) must not apply `gain` a second time
    }

    fn take_samples(&mut self, mclk: i64) -> Vec<(i32, i32)> {
        Psg::take_samples(self, mclk)
    }
}
