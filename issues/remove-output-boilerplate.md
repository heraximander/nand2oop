# Remove output struct boilerplate from HDL

## Type

Feature

## Status

Open

## Description

Currently the return value from a chip definition function must be a struct
which implements the `StructuredData` trait. While this is useful for complex
chips, it adds significant boilerplate for chips which only return a single value or array. For example:

```rust
UnaryChipOutput {
    out: nand.into(),
}
```

There are a few things we could do here:

1. Allow chip definition functions to also return single values or arrays, not
   just structs
1. Implement the `Into<UnaryChipOutput>` trait, so that returning a single 
   value looks like:
```rust
nand.into()
```
1. Add a `::new()` method to `StructuredData` structs which constructs structs
   without requiring field names, eg:
```rust
UnaryChipOutput::nand.into()
```
  The `UnaryChipOutput` part could also be inferred by generics if you change
  the struct definition