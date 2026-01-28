use std::{fmt::{self, Display, Write}, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use formatter_builder::FormatterBuilder;

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
        Display::fmt(&self.0, f)?;
        write!(f, ", {} ->", self.1)?;
        Display::fmt(&self.2, f)
    }
}
impl fmt::Octal for Foo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        FormatterBuilder::from_formatter_lossy(f).with(f, |f| {
            write!(f, "Foo: ")?;
            Display::fmt(&self.0, f)?;
            write!(f, ", {} ->", self.1)?;
            Display::fmt(&self.2, f)
        })
    }
}

fn rand_state() -> oorandom::Rand32 {
    black_box(oorandom::Rand32::new(black_box(std::process::id()) as u64))
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("default", |b| {
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
    c.bench_function("builder-from-lossy", |b| {
        let mut seed = rand_state();
        b.iter(|| {
            let foo = Foo(seed.rand_u32(), seed.rand_i32(), seed.rand_u32());
            let _ = write!(NullWriter, "{foo:#o}");
            let _ = write!(NullWriter, "{foo:+o}");
            let _ = write!(NullWriter, "{foo:05o}");
            let _ = write!(NullWriter, "{foo:+03o}");
            let _ = write!(NullWriter, "{foo:0>+03o}");
            let _ = write!(NullWriter, "{foo:0>+3o}");
            let _ = write!(NullWriter, "{foo:03.2o}");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
