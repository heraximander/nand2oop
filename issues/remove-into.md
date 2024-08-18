# Remove `::into()` boilerplate

## Type

Feature

## Status

Open

## Description

Since the implementation of [Into traits](./add-into-implementations-inputs.md)
we have a lot of `.into()` method calls littering the code:

```rust
let initial_nor = Or::new(
    alloc,
    in_[0].into(),
    in_[1].into()
);
```

Would it be possible to remove the `.into()`s by making `Chip::new()` accept
`impl Into<Input>` instead, and putting the `.into()` in the `::new()` function?