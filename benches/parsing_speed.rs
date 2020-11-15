use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use redis_parser::resp2::{parse as parse2, Resp2Type};

fn resp_2_parse(mut data: &[u8]) -> Vec<Resp2Type> {
    let it = std::iter::from_fn(move || {
        match parse2(data) {
            // when successful, a nom parser returns a tuple of
            // the remaining input and the output value.
            // So we replace the captured input data with the
            // remaining input, to be parsed on the next call
            Ok((i, o)) => {
                data = i;
                Some(o)
            }
            _ => None,
        }
    });

    it.collect()
}

fn run_2_parser(c: &mut Criterion) {
    let raw_text = include_str!("resp2_data.txt");
    let replaced = raw_text.replace('\n', "\r\n");
    let data = replaced.as_bytes();

    let mut group = c.benchmark_group("res2");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("res2", |b| b.iter(|| resp_2_parse(black_box(data))));
    group.finish();
}

criterion_group!(benches, run_2_parser);
criterion_main!(benches);
