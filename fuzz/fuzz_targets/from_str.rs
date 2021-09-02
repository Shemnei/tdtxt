#![no_main]
use std::str::FromStr;

use libfuzzer_sys::fuzz_target;
use tdtxt::Task;

fuzz_target!(|data: &[u8]| {
	if let Ok(s) = std::str::from_utf8(data) {
		let _ = Task::from_str(s);
	}
});
