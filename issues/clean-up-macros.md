# Clean up macros

## Type

Tech debt

## Status

Open

## Description

The `#[chip]` and `[define(StructuredData)] macros aren't very robust. Some
issues are:

* They mostly require generic parameters defined by the user to have a
  particular name, mostly `T`
* `::get_arity()` was defined so the `#[chip]` macro could get the output size
  at compile time, but as it isn't a part of the `StructuredData` interface it
  creates an implicit dependency on `#[define(StructuredData)]` macro which
  isn't ideal

We should do a full audit of this macros, create tests for edge cases and fix
any issues that arise.
