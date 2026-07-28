#![no_main]
use kime_fuzz::diff::DiffOp;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|ops: Vec<DiffOp>| {
    kime_fuzz::diff::run_diff(&ops);
});
