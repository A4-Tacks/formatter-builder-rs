use std::{fmt::{self, Alignment, Display, Write}, hint::black_box, thread::spawn, time::Instant};

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

use formatter_builder::{Fill, FormatterBuilder, Sign};

fn rand_state() -> oorandom::Rand32 {
    black_box(oorandom::Rand32::new(black_box(std::process::id()) as u64))
}

fn main() {
    let mut seed = rand_state();
    spawn(move || {
        let start = Instant::now();
        for _ in 0..10000000 {
            let foo = Foo(seed.rand_u32(), seed.rand_i32(), seed.rand_u32());
            let _ = FormatterBuilder::new().alternate(true).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign(Sign::Plus).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign_aware_zero_pad(true).width(5).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign(Sign::Plus).sign_aware_zero_pad(true).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().align(Alignment::Right).fill(Fill::Zero).sign(Sign::Plus).sign_aware_zero_pad(true).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().align(Alignment::Right).fill(Fill::Zero).sign(Sign::Plus).width(3).with(NullWriter, |f| foo.fmt(f));
            let _ = FormatterBuilder::new().sign_aware_zero_pad(true).width(3).precision(2).with(NullWriter, |f| foo.fmt(f));
        }
        let end = start.elapsed();
        dbg!(end);
    }).join().unwrap();
}
