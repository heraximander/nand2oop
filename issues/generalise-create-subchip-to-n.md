# Generalise `create_subchip()` to operate over N chips

## Type

Feature

## Status

Open

## Description

`create_subchip()` will currently only create two mutually dependent chips. We
could generalise this to operate over _N_ chips. This would probably be achieved
by some type of code generation.

This is low priority as the current functionality is sufficient for the latches
I am creating.
