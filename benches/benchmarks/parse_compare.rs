use std::str::FromStr;

use criterion::{criterion_group, BenchmarkId, Criterion};

const GITHUB_EXAMPLE: &str = "x (A) 2016-05-20 2016-04-30 measure space for \
                              +chapelShelving @chapel due:2016-05-30";

fn compare_parse(c: &mut Criterion) {
	let mut group = c.benchmark_group("Parse Compare");

	group.bench_with_input(
		BenchmarkId::new("tdtxt", &GITHUB_EXAMPLE),
		&GITHUB_EXAMPLE,
		|b, s| b.iter(|| tdtxt::Task::from_str(s)),
	);

	group.bench_with_input(
		BenchmarkId::new("todotxt", &GITHUB_EXAMPLE),
		&GITHUB_EXAMPLE,
		|b, s| b.iter(|| todotxt::Task::from_str(s)),
	);

	group.bench_with_input(
		BenchmarkId::new("todo_txt", &GITHUB_EXAMPLE),
		&GITHUB_EXAMPLE,
		|b, s| b.iter(|| todo_txt::Task::from_str(s)),
	);

	group.finish();
}

criterion_group!(benches, compare_parse);
