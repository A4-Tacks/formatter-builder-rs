use std::{fmt::{self, Alignment, Display, Write}, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use formatter_builder::{Fill, FormatterBuilder, Sign};

struct NullWriter;
impl Write for NullWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        black_box(s);
        black_box(Ok(black_box(())))
    }
}

struct Foo(u32, i32, u32);
impl Display for Foo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Foo: ")?;
        self.0.fmt(f)?;
        write!(f, ", {} ->", self.1)?;
        self.2.fmt(f)
    }
}

fn rand_state() -> oorandom::Rand32 {
    black_box(oorandom::Rand32::new(black_box(std::process::id()) as u64))
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_args", |b| {
        let mut seed = rand_state();
        b.iter(|| {
            let foo = Foo(seed.rand_u32(), seed.rand_i32(), seed.rand_u32());
            let _ = write!(NullWriter, "{foo:#}");
            let _ = write!(NullWriter, "{foo:+}");
            let _ = write!(NullWriter, "{foo:05}");
            let _ = write!(NullWriter, "{foo:+03}");
            let _ = write!(NullWriter, "{foo:0>+03}");
            let _ = write!(NullWriter, "{foo:0>+3}");
            let _ = write!(NullWriter, "{foo:03.2}");
        })
    });
    c.bench_function("builder", |b| {
        let mut seed = rand_state();
        b.iter(|| {
            let foo = Foo(seed.rand_u32(), seed.rand_i32(), seed.rand_u32());
            let _ = FormatterBuilder::new().alternate(true).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign(Sign::Plus).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign_aware_zero_pad(true).width(5).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign(Sign::Plus).sign_aware_zero_pad(true).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().align(Alignment::Right).fill(Fill::Zero).sign(Sign::Plus).sign_aware_zero_pad(true).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().align(Alignment::Right).fill(Fill::Zero).sign(Sign::Plus).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign_aware_zero_pad(true).width(3).precision(2).with(NullWriter, |f| foo.fmt(f));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
