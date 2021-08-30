use std::str::FromStr;

use criterion::{criterion_group, BenchmarkId, Criterion, Throughput};

fn compare_parse(c: &mut Criterion) {
	let mut group = c.benchmark_group("Parse Compare");

	for input in [
		"x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving \
		 @chapel due:2016-05-30",
		"(A) Thank Mom for the meatballs @phone",
		"(B) Schedule Goodwill pickup +GarageSale @phone",
		"Post signs around the neighborhood +GarageSale",
		"@GroceryStore Eskimo pies",
	] {
		group.throughput(Throughput::Bytes(input.len() as u64));

		group.bench_with_input(
			BenchmarkId::new("tdtxt", input),
			input,
			|b, s| b.iter(|| tdtxt::Task::from_str(s)),
		);

		group.bench_with_input(
			BenchmarkId::new("todotxt", input),
			input,
			|b, s| b.iter(|| todotxt::Task::from_str(s)),
		);

		group.bench_with_input(
			BenchmarkId::new("todo_txt", input),
			input,
			|b, s| b.iter(|| todo_txt::Task::from_str(s)),
		);
	}

	group.finish();
}

criterion_group!(benches, compare_parse);
