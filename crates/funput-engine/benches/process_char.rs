//! End-to-end keystroke latency/throughput for `funput-engine`.
//!
//! Drives the full engine path (transform + word-boundary handling + English
//! restore) by replaying a multi-sentence paragraph, exactly as a platform shell
//! would. Reports ns/keystroke and keystrokes/second.
//!
//! Run: `cargo bench -p funput-engine`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use funput_core::InputMethod;
use funput_engine::Engine;

const TELEX: &str =
    "Tooi yeeu tieesng Vieejt. Hoom nay troiwf nuwowcs ddepj. Ban cos khoeer khoong?";
const VNI: &str = "To6i ye6u tie6ng1 Vie6t5. Ho6m nay tro7i2 nu7o7c1 dde9p5. Ban co1 khoe3 kho6ng?";

fn run(input: &str, method: InputMethod) {
    let mut engine = Engine::new();
    engine.set_method(method);
    for k in input.chars() {
        let _ = engine.process_char(k);
    }
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_char");
    for (name, input, method) in [
        ("telex", TELEX, InputMethod::Telex),
        ("vni", VNI, InputMethod::Vni),
    ] {
        group.throughput(Throughput::Elements(input.chars().count() as u64));
        group.bench_function(BenchmarkId::new("paragraph", name), |b| {
            b.iter(|| run(black_box(input), method))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
