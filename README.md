# reporting [![Crates.io Version](https://img.shields.io/crates/v/reporting)](https://crates.io/crates/reporting) [![docs.rs](https://img.shields.io/docsrs/reporting)](https://docs.rs/reporting/latest/reporting/index.html)
Simple diagnostic reporting for compilers.

<img src="sample.svg">

```rust
use reporting::{error, note, File, Location, Renderer, Styles};

fn main() {
    let file = File::new("test.txt", "import stds;");
    let styles = Styles::styled();

    print!(
        "{}",
        Renderer::new(
            &styles,
            &[
                error!("Could not find package `{}`", "stds")
                    .location(Location::new(file.clone(), 7)),
                note!("Perhaps you meant `std`?")
            ]
        )
    )
}
```