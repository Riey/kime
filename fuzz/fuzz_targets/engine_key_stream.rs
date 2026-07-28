#![no_main]
use kime_fuzz::Op;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u8, Vec<Op>)| {
    let (preset, ops) = input;
    kime_fuzz::run_ops(preset, &ops);
});
