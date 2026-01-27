#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

use core::fmt::{self, Write, Formatter};
pub use core::fmt::{Alignment};

/// The signedness of a [`Formatter`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    /// Represents the `+` flag.
    Plus,
    /// Represents the `-` flag.
    Minus,
}

/// [`Formatter`] fill character.
///
/// Due to some limitations, only a small number of characters are supported.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Character `'0'`
    Zero,
    /// Character `' '`
    Space,
}

/// [`Formatter`] safe builder.
///
#[doc = include_str!("../README.md")]
#[derive(Debug, Default)]
pub struct FormatterBuilder {
    sign: Option<Sign>,
    sign_aware_zero_pad: bool,
    alternate: bool,
    fill_align: Option<(Option<Fill>, Alignment)>,
    width: Option<u16>,
    precision: Option<u16>,
}

macro_rules! pack {
    ($name:ident $bang:tt $args:tt $($t:tt)*) => {
        pack!(@run {$name $bang $args} () $($t)*)
    };
    (@run $cfg:tt $coll:tt $arg:tt $($t:tt)*) => {
        pack!(@run $cfg ($coll $arg) $($t)*)
    };
    (@run $cfg:tt $coll:tt) => {
        pack!(@back $cfg () $coll)
    };
    (@back $cfg:tt $coll:tt ($a:tt $b:tt)) => {
        pack!(@back $cfg ($b $coll) $a)
    };
    (@back {$name:tt $bang:tt ($($args:tt)*)} $coll:tt ()) => {
        $name $bang ($($args)* $coll)
    };
}

macro_rules! builder {
    (@run[$self:tt $f:tt $caps:tt $cfg:tt]
        (($field:ident [$($pat:pat => $lit:literal $($cap:ident)?),*]) $rec:tt)
    ) => {
        match $self.$field {
            $(
                $pat => builder!(@run[$self (concat!$f, $lit) ($caps $($cap)?) $cfg] $rec),
            )*
        }
    };
    (@run[$self:tt $f:tt ((((((())))) $($cap1:tt)?) $($cap2:tt)?) $cfg:tt] ()) => {
        //                 ^ 之后的代码, 分支数每多一个就添加一个括号
        builder!(@fin[$self $f ($($cap1)? $($cap2)?) $cfg])
    };
    (@fin[$self:tt $f:tt ($($caps:tt)*) {
        $writer:ident,
        $format_with:ident
    }]) => {
        //compile_error!(concat!("{:", concat!$f, "}"))
        write!($writer, concat!("{:", concat!$f, "}"), $format_with, $($caps = $caps,)*)?
    };
    ($self:ident, $writer:ident, $format_with:ident {
        $($field:ident [$($pat:pat => $lit:literal $($cap:ident)?),* $(,)?]),+ $(,)?
    }) => {
        pack!(builder!(@run[$self ("") () {
            $writer,
            $format_with
        }]) $(($field [$($pat => $lit $($cap)?),*]))*)
    };
}

impl FormatterBuilder {
    pub fn with<W, F>(&self, mut writer: W, f: F) -> fmt::Result
    where
        W: Write,
        F: FnOnce(&mut Formatter<'_>) -> fmt::Result,
    {
        let width = self.width.unwrap_or(0) as usize;
        let precision = self.precision.unwrap_or(0) as usize;

        let format_with = FormatWith(Some(f).into());

        builder!(self, writer, format_with {
            fill_align [
                Some((None, Alignment::Left)) => "<",
                Some((None, Alignment::Right)) => ">",
                Some((None, Alignment::Center)) => "^",
                Some((Some(Fill::Zero), Alignment::Left)) => "0<",
                Some((Some(Fill::Zero), Alignment::Right)) => "0>",
                Some((Some(Fill::Zero), Alignment::Center)) => "0^",
                Some((Some(Fill::Space), Alignment::Left)) => " <",
                Some((Some(Fill::Space), Alignment::Right)) => " >",
                Some((Some(Fill::Space), Alignment::Center)) => " ^",
                None => "",
            ],
            sign [
                Some(Sign::Plus) => "+",
                Some(Sign::Minus) => "-",
                None => "",
            ],
            alternate [
                true => "#",
                false => "",
            ],
            sign_aware_zero_pad [
                true => "0",
                false => "",
            ],
            width [
                Some(_) => "width$" width,
                None => "",
            ],
            precision [
                Some(_) => ".precision$" precision,
                None => "",
            ],
        });

        Ok(())
    }
}

impl FormatterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sign(&mut self, sign: impl Into<Option<Sign>>) -> &mut Self {
        self.sign = sign.into();
        self
    }

    pub fn sign_aware_zero_pad(&mut self, sign_aware_zero_pad: bool) -> &mut Self {
        self.sign_aware_zero_pad = sign_aware_zero_pad;
        self
    }

    pub fn alternate(&mut self, alternate: bool) -> &mut Self {
        self.alternate = alternate;
        self
    }

    /// # Panics
    ///
    /// - panic when [`align`](FormatterBuilder::align) is unset
    #[track_caller]
    pub fn fill(&mut self, fill: impl Into<Option<Fill>>) -> &mut Self {
        if let Some(fill_char) = fill.into() {
            self.fill_align.as_mut().expect(".fill must setted align").0 = Some(fill_char);
        }
        self
    }

    pub fn align(&mut self, align: impl Into<Option<Alignment>>) -> &mut Self {
        if let Some(alignment) = align.into() {
            self.fill_align.get_or_insert((None, alignment)).1 = alignment;
        }
        self
    }

    pub fn width(&mut self, width: impl Into<Option<u16>>) -> &mut Self {
        self.width = width.into();
        self
    }

    pub fn precision(&mut self, precision: impl Into<Option<u16>>) -> &mut Self {
        self.precision = precision.into();
        self
    }
}

struct FormatWith<F>(core::cell::Cell<Option<F>>)
where
    F: FnOnce(&mut Formatter<'_>) -> fmt::Result,
;

impl<F> fmt::Display for FormatWith<F>
where
    F: FnOnce(&mut Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.take().unwrap()(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_work() {
        let mut out = String::new();
        FormatterBuilder::new()
            .with(&mut out, |f| {
                write!(f, "foo")?;
                write!(f, "bar")?;
                Ok(())
            }).unwrap();
        assert_eq!(out, "foobar");
    }
}
