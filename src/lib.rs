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

// NOTE: 如果要添加更多字符, 是一个 break,
// 并且哪怕声明非穷尽, 宏代码量也会爆炸, 体验会很大影响, 所以需要非同一个版本

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
impl Fill {
    pub fn as_char(self) -> char {
        match self {
            Self::Zero => '0',
            Self::Space => ' ',
        }
    }
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

    /// Format like `{:+}` and `{:-}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// use std::fmt::Display;
    /// let mut writter = String::new();
    /// FormatterBuilder::new().sign(Sign::Plus).with(&mut writter, |f| {
    ///     2i32.fmt(f);
    ///     Ok(())
    /// }).unwrap();
    /// assert_eq!(writter, "+2");
    /// ```
    pub fn sign(&mut self, sign: impl Into<Option<Sign>>) -> &mut Self {
        self.sign = sign.into();
        self
    }

    /// Format like `{:0}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// # let writter = String::new();
    /// FormatterBuilder::new().sign_aware_zero_pad(true).with(writter, |f| {
    ///     assert!(f.sign_aware_zero_pad());
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn sign_aware_zero_pad(&mut self, sign_aware_zero_pad: bool) -> &mut Self {
        self.sign_aware_zero_pad = sign_aware_zero_pad;
        self
    }

    /// Format like `{:#}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// # let writter = String::new();
    /// FormatterBuilder::new().alternate(true).with(writter, |f| {
    ///     assert!(f.alternate());
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn alternate(&mut self, alternate: bool) -> &mut Self {
        self.alternate = alternate;
        self
    }

    /// Format like `{:0>}` `{: ^}` etc
    ///
    /// # Panics
    ///
    /// - panic when [`align`](FormatterBuilder::align) is unset
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// # use Alignment::*;
    /// # let writter = String::new();
    /// FormatterBuilder::new().align(Left).fill(Fill::Space).with(writter, |f| {
    ///     assert_eq!(f.fill(), ' ');
    ///     Ok(())
    /// }).unwrap();
    /// ```
    #[track_caller]
    pub fn fill(&mut self, fill: impl Into<Option<Fill>>) -> &mut Self {
        if let Some(fill_char) = fill.into() {
            self.fill_align.as_mut().expect(".fill must setted align").0 = Some(fill_char);
        }
        self
    }

    /// Format like `{:<}` and  `{:^}` and `{:>}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// # use Alignment::*;
    /// # let writter = String::new();
    /// FormatterBuilder::new().align(Center).with(writter, |f| {
    ///     assert_eq!(f.align(), Some(Center));
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn align(&mut self, align: impl Into<Option<Alignment>>) -> &mut Self {
        if let Some(alignment) = align.into() {
            self.fill_align.get_or_insert((None, alignment)).1 = alignment;
        }
        self
    }

    /// Format like `{:3}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// use std::fmt::Display;
    /// let mut writter = String::new();
    /// FormatterBuilder::new().width(3).with(&mut writter, |f| {
    ///     2i32.fmt(f);
    ///     Ok(())
    /// }).unwrap();
    /// assert_eq!(writter, "  2");
    /// ```
    pub fn width(&mut self, width: impl Into<Option<u16>>) -> &mut Self {
        self.width = width.into();
        self
    }

    /// Format like `{:.3}`
    ///
    /// # Examples
    ///
    /// ```
    /// # use formatter_builder::*;
    /// use std::fmt::Display;
    /// let mut writter = String::new();
    /// FormatterBuilder::new().precision(3).with(&mut writter, |f| {
    ///     2f32.fmt(f);
    ///     Ok(())
    /// }).unwrap();
    /// assert_eq!(writter, "2.000");
    /// ```
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
    use Alignment::*;
    use Fill::*;

    const W: String = String::new();

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

    #[test]
    fn align() {
        let aligns = [ Left, Right, Center ];
        for align in aligns {
            FormatterBuilder::new()
                .align(align)
                .with(W, |f| {
                    assert_eq!(f.align(), Some(align));
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn fill() {
        let fills = [Zero, Space];
        for fill in fills {
            FormatterBuilder::new()
                .align(Left)
                .fill(fill)
                .with(W, |f| {
                    assert_eq!(f.fill(), fill.as_char());
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn alternate() {
        for alt in [true, false] {
            FormatterBuilder::new()
                .alternate(alt)
                .with(W, |f| {
                    assert_eq!(f.alternate(), alt);
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn width() {
        for width in [None, Some(0), Some(1), Some(2), Some(4), Some(256)] {
            FormatterBuilder::new()
                .width(width)
                .with(W, |f| {
                    assert_eq!(f.width(), width.map(Into::into));
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn precision() {
        for precision in [None, Some(0), Some(1), Some(2), Some(4), Some(256)] {
            FormatterBuilder::new()
                .precision(precision)
                .with(W, |f| {
                    assert_eq!(f.precision(), precision.map(Into::into));
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn sign_aware_zero_pad() {
        for sazp in [true, false] {
            FormatterBuilder::new()
                .sign_aware_zero_pad(sazp)
                .with(W, |f| {
                    assert_eq!(f.sign_aware_zero_pad(), sazp);
                    Ok(())
                }).unwrap();
        }
    }

    #[test]
    fn sign() {
        // Sign::Minus unused
        for (sign, exp) in [(Sign::Plus, "+1")] {
            let mut out = String::new();
            FormatterBuilder::new()
                .sign(sign)
                .with(&mut out, |f| {
                    std::fmt::Display::fmt(&1, f)?;
                    Ok(())
                }).unwrap();
            assert_eq!(out, exp);
        }
    }
}
