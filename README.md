The safe and stable builder for [`fmt::Formatter`]

Due to unstable [`Formatter::new`], uses this crate to dynamically build a [`Formatter`].

# Examples

```rust
use formatter_builder::{FormatterBuilder, Fill, Sign, Alignment};

let mut output = String::new();
let n = 6.23;
formatter_builder::FormatterBuilder::new()
    .width(8)
    .align(Alignment::Right)
    .precision(3)
    .fill(Fill::Zero)
    .with(&mut output, |f| {
        std::fmt::Display::fmt(&n, f)
    })
    .unwrap();
assert_eq!(output, "0006.230")
```
