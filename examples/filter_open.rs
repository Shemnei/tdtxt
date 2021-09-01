use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use tdtxt::{State, Task};

/// Prints an error message and aborts the program.
macro_rules! abort {
    () => (eprint!("\n"));
    (
		$( $arg:tt )*
	) => ({
        eprint!("ERROR: ");
        eprintln!($($arg)*);
        ::std::process::exit(1);
    })
}

fn main() {
	let args = std::env::args_os();

	if args.len() == 2 {
		// Get the path from the arguments
		let path = args.skip(1).next().unwrap_or_else(|| print_help_exit());

		// Create a buffered reader to read line by line
		let buf_reader =
			BufReader::new(File::open(&path).unwrap_or_else(|err| {
				abort!("Failed to read file at path `{:?}`: {}", path, err)
			}));

		// Parse lines as task and filter out errors and closed tasks
		// Collect so the tasks can be sorted
		let mut open_tasks = buf_reader
			.lines()
			.filter_map(|line| Task::from_str(&line.ok()?).ok())
			.filter(|task| task.state == State::Open)
			.collect::<Vec<_>>();

		// Order by priority and the creation date
		open_tasks.sort_by(task_ord);

		// Print open tasks
		println!("=== OPEN TASKS ===");
		for task in open_tasks {
			println!("  - {}", task);
		}
	} else {
		print_help_exit();
	}
}

/// Prints a help text and then exits the program.
fn print_help_exit() -> ! {
	println!("cargo run --example filter_open -- PATH");
	std::process::exit(1);
}

/// Orders tasks by priority (highest to lowest) and then by creation date (older to newest/none).
fn task_ord(a: &Task, b: &Task) -> Ordering {
	// Invert order, so that the highest priority is on top
	let prio_cmp = b.priority().cmp(&a.priority());

	if prio_cmp == Ordering::Equal {
		match (
			a.date_compound().map(|dc| dc.date_created()),
			b.date_compound().map(|dc| dc.date_created()),
		) {
			(Some(adc), Some(bdc)) => adc.cmp(bdc),
			(None, Some(_)) => Ordering::Greater,
			(Some(_), None) => Ordering::Less,
			(None, None) => Ordering::Equal,
		}
	} else {
		prio_cmp
	}
}
