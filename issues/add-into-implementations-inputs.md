# Add `::into()` implementations for `ChipInput`, `ChipOutput` to `Input` enum

## Type

Feature

## Status

Complete

## Description

Currently the user has to wrap any `ChipInput`s in an `Input` enum before
passing the value to a chip constructor. They do this either by:

```rust
Input::ChipInput(in1)
```

for single values or

```rust
input.map(Input::ChipInput)
```

for arrays. This is way too much boilerplate, is a pain to write and fills up
the file with redundant information. A way to simplify this would be to add an
`::into()` trait implementation to `Input` enum, to get something like:

```rust
in1.into()
```

and

```rust
input.into()
```

Note that the latter might not be easy to implement in such a way with the
existing `Into` trait, so we might have to make our own similarly named one,
eg `::elem_into()`. 