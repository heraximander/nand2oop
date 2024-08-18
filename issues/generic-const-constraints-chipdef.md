# Add constant generics for chip inputs and outputs

## Type

Feature

## Status

Open

## Description

Chip definitions are currently tied to a certain input-output arity. For some
chip definitions, for example a multi-bit adder, the definition can easily be
genericised to be of input and output size `N`. This would mean we wouldn't have
to redefine chips for different input and output sizes.

This also holds true for `StructuredData` structs.