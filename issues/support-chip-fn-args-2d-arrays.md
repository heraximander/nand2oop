# Support 2D/nD arrays as arguments to chip functions

## Type

Feature

## Status

Open

## Description

Currently we only allow chip functions to accept inputs or 1D arrays of inputs.
Representing 2D arrays, as in the below function, is cumbersome:

```rust
#[chip]
fn mux16x8<'a>(
    alloc: &'a Bump,
    in1: [&'a ChipInput<'a>; 16],
    in2: [&'a ChipInput<'a>; 16],
    in3: [&'a ChipInput<'a>; 16],
    in4: [&'a ChipInput<'a>; 16],
    in5: [&'a ChipInput<'a>; 16],
    in6: [&'a ChipInput<'a>; 16],
    in7: [&'a ChipInput<'a>; 16],
    in8: [&'a ChipInput<'a>; 16],
    sel: [&'a ChipInput<'a>; 3],
) -> ArrayLen16<ChipOutputType<'a>> {
    ...
}
```

We might want to instead support:

```rust
#[chip]
fn mux16x8<'a>(
    alloc: &'a Bump,
    in_: [[&'a ChipInput<'a>; 16]; 8],
    sel: [&'a ChipInput<'a>; 3],
) -> ArrayLen16<ChipOutputType<'a>> {
    ...
}
```

This would massively reduce the amount of boilerplate in some areas, most
notably in the RAM chip hierarchical constructions.
