// SPDX-License-Identifier: MIT
//! Frames per second of this board on a card, nothing logged or digested.
//!
//!     pce-bench IMAGE [--frames N] [options of trace::Setup]
//!
//! Speed is not this crate's goal (the fast fork's); this is the figure to measure a fork
//! against. `scripts/bench.py` times the reference the same way.

use std::time::Instant;

use pcengine_rust::trace::Setup;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let setup = match Setup::parse(&args) {
        Ok(setup) => setup,
        Err(error) => {
            eprintln!("pce-bench: {error}");
            std::process::exit(1);
        }
    };
    let mut system = setup.build().expect("a board");
    let began = Instant::now();
    let mut samples = 0;
    for _ in 0..setup.frames {
        setup.apply_presses(&mut system);
        system.run_frame();
        samples += system.board_mut().take_audio().len();
    }
    let seconds = began.elapsed().as_secs_f64();
    println!(
        "{} frames in {:.3} s: {:.1} frames per second ({} audio samples, mclk {})",
        setup.frames,
        seconds,
        setup.frames as f64 / seconds,
        samples,
        system.board().mclk
    );
}
