use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use redis_parser::resp2::{parse as parse2, Resp2Type};

fn resp_2_parse(mut data: &[u8]) {
    loop {
        match parse2(data) {
            Ok((i, o)) => {
                data = i;
                black_box(o);
            }
            _ => {
                break;
            }
        }
    }
}

fn run_2_parser(c: &mut Criterion) {
    let raw_text = include_str!("resp2_data.txt");
    let replaced = raw_text.replace('\n', "\r\n");
    let repeated = replaced.repeat(30);
    let data = repeated.as_bytes();

    let mut group = c.benchmark_group("res2");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("res2", |b| b.iter(|| resp_2_parse(black_box(data))));
    group.finish();
}

criterion_group!(benches, run_2_parser);
criterion_main!(benches);
