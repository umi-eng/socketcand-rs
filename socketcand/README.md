# socketcand

A socketcand parser implementation built with [`nom`](https://docs.rs/nom). Designed for `no_std` environments and implements [`embedded-can`](https://docs.rs/embedded-can) traits.

```rust
let input = "< send 123 0 >";
let (_, result) = command(input).unwrap();
println!("{:?}", result);
```

## Optional features

- `defmt-03`: Derive `defmt::Format` from `defmt` 0.3 for enums and structs.
