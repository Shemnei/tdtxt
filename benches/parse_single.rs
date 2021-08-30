use std::str::FromStr;

use criterion::{
	black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use tdtxt::Task;

const GITHUB_EXAMPLE: &str = "x (A) 2016-05-20 2016-04-30 measure space for \
                              +chapelShelving @chapel due:2016-05-30";

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_with_input(
		BenchmarkId::new("parse_github_example", GITHUB_EXAMPLE),
		&GITHUB_EXAMPLE,
		|b, s| b.iter(|| Task::from_str(s)),
	);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
