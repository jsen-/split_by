[![Latest Version](https://img.shields.io/crates/v/split_by.svg)](https://crates.io/crates/split_by)

# split_by
Split anything implementing Read trait by multiple sequences of bytes

[Documentation](https://docs.rs/split_by)

## Quick start

Add this to Cargo.toml, under `[dependencies]`:

```toml
split_by = "0.1"
```

## Usage
```rust
extern crate split_by;
use split_by::{AcAutomaton, SplitBy}
use std::fs::File;

fn main() {
    let ac = AcAutomaton::new(vec!["<some pattern>"]); 
    for section in File::open("path/to/file").unwrap().split_by(&ac) {
        // do something with the bytes found bytes between patterns
    }
}
```