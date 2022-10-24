# btf-derive
## Description
Provides various macros for the btf crate.

## Example
```rust
use btf::{AddToBtf, BtfTypes};
use btf_derive::AddToBtf;

#[derive(AddToBtf)]
struct MyNewType {
    pub a: u32,
    pub b: u64,
}

let mut btf = BtfTypes::default();
MyNewType::add_to_btf(&mut btf).expect("Failed to add type.");
```
