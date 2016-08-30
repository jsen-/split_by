[![Latest Version](https://img.shields.io/crates/v/split_by.svg)](https://crates.io/crates/split_by)

# split_by
Split anything implementing Read trait by multiple sequences of bytes

[Documentation](https://docs.rs/split_by)

## Quick start

Add this to Cargo.toml, under `[dependencies]`:

```toml
split_by = "0.2"
```

## Usage
```rust
extern crate split_by;
use split_by::{AcAutomaton, SplitBy}
use std::fs::File;

fn main() {
    for section in File::open("path/to/file").unwrap().split_by(&AcAutomaton::new(vec!["<some pattern>"])) {
        let bytes = section.expect("read error occurred");
        // do something with the bytes found between patterns
    }
}
```