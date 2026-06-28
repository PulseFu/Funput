//! End-to-end per-keystroke latency across the C FFI boundary.
//!
//! This is the real work a platform shell (Swift IMKit, Windows hook, ibus/fcitx5)
//! pays for every key — the layer the pure-core bench skips:
//!   1. `funput_process_char` — run the engine and return the [`FunputResult`] POD
//!      (~268 bytes) **by value** across the ABI.
//!   2. `funput_buffer` — copy the composed text back out (UTF-32) to render the
//!      marked/underlined composition.
//!
//! It does NOT include OS event delivery or the app's own text render — those are
//! not Funput's code and can't be measured reproducibly. So this is "keystroke →
//! composed text available to the platform", i.e. Funput's full contribution to
//! per-keystroke latency.
//!
//! Run: `cargo bench -p funput-ffi --bench latency`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use funput_ffi::{
    funput_buffer, funput_clear, funput_engine_free, funput_engine_new, funput_process_char,
    funput_set_method, CHARS_CAP, FunputEngine,
};

const TELEX: &str =
    "Tooi yeeu tieesng Vieejt. Hoom nay troiwf nuwowcs ddepj. Ban cos khoeer khoong?";
const VNI: &str = "To6i ye6u tie6ng1 Vie6t5. Ho6m nay tro7i2 nu7o7c1 dde9p5. Ban co1 khoe3 kho6ng?";

/// One full keystroke as a shell does it: process the key, then read the composed
/// buffer back to render it.
fn keystroke(engine: *mut FunputEngine, ch: char, out: &mut [u32; CHARS_CAP]) {
    let result = unsafe { funput_process_char(engine, ch as u32) };
    let n = unsafe { funput_buffer(engine, out.as_mut_ptr(), CHARS_CAP) };
    black_box((result.action, result.count, n));
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_latency");
    for (name, input, method) in [("telex", TELEX, 0u8), ("vni", VNI, 1u8)] {
        let engine = funput_engine_new();
        unsafe { funput_set_method(engine, method) };
        let mut out = [0u32; CHARS_CAP];

        group.throughput(Throughput::Elements(input.chars().count() as u64));
        group.bench_function(BenchmarkId::new("process+render", name), |b| {
            b.iter(|| {
                unsafe { funput_clear(engine) };
                for ch in black_box(input).chars() {
                    keystroke(engine, ch, &mut out);
                }
            })
        });

        unsafe { funput_engine_free(engine) };
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
