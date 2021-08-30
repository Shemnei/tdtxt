use criterion::criterion_main;

mod benchmarks;

criterion_main! {
	benchmarks::parse_single::benches,
	benchmarks::parse_compare::benches,
}
