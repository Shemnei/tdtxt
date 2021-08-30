use std::str::FromStr;

use criterion::{criterion_group, BenchmarkId, Criterion};
use tdtxt::Task;

const GITHUB_EXAMPLE: &str = "x (A) 2016-05-20 2016-04-30 measure space for \
                              +chapelShelving @chapel due:2016-05-30";

fn criterion_benchmark(c: &mut Criterion) {
	c.bench_with_input(
		BenchmarkId::new("parse_single", GITHUB_EXAMPLE),
		&GITHUB_EXAMPLE,
		|b, s| b.iter(|| Task::from_str(s)),
	);
}

criterion_group!(benches, criterion_benchmark);
