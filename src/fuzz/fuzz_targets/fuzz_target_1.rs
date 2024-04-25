#![no_main]
#[macro_use] extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
});
