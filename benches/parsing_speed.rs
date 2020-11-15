use criterion::{black_box, criterion_group, criterion_main, Criterion};
use redis_parser::resp2::{parse as parse2, Resp2Type};

fn resp_2_parse() -> Vec<Resp2Type> {
    let bytes = include_bytes!("resp2_data.txt");
    let mut result = vec![];
    loop {
        let (bytes, value) = parse2(bytes);
        result.append(value);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

