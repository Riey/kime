#![no_main]
//! Fuzz the GTK immodule commit/emit/reset protocol mirror (fuzz/src/frontend.rs)
//! against a hostile client that re-enters reset() from signal handlers.

use kime_fuzz::frontend::{run_frontend, FrontendInput};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: FrontendInput| {
    run_frontend(&input);
});
