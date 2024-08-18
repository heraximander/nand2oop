# Remove input struct boilerplate from HDL

## Type

Feature

## Status

Complete

## Description

Since the implementation of [structured inputs](../docs/adr/structured-inputs-outputs.md)
there has been a fair bit of boilerplate when `::new()`ing up chips:

```rust
let initial_nor = Or::new(
    alloc,
    OrInputs {
        in1: in_[0].into(),
        in2: in_[1].into(),
    },
);
```

We could simplify this to the much more readable:

```rust
let initial_nor = Or::new(alloc, in_[0].into(), in_[1].into());
```

without struct or field names. We would need to retain a way of constructing
chips using an input struct so `Machine` can construct chips, perhaps by way
of renaming the current `::new()` implementation to `::from()`.