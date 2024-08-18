# Allowing cycles in HDL code

## Background

When the HDL code was initially written I only implemented capabilities for
combinatorial components. I now need to implement sequential components which
I have decided to do through [modelling cycles in HDL](./modelling-sequential-components.md).
Note that HDL currently tries to have all its inputs statically verifiable, ie
the compiler checks that each component has enough inputs. This is currently
ensured by the chip constructor, which requires a value for each of its inputs,
such that the chip cannot be constructed without enough inputs. The problem for
cyclic chips is that we cannot supply enough inputs to each chip during its
initial construction because we haven't yet created it, as it is also dependent
on the chip we are trying to create.

## Problem

How can I implement cycles in HDL while maintaining static verification of chip
inputs?

## Solutions

### Allow chips to be created with fewer than necessary inputs and fail at run time

We could just allow chips to be created and instantiated later, without any
compile time checking that the user has supplied all the required inputs.

Pros:
* Simple to implement
* Not that hard to visually verify, especially we aren't going to write many
  cyclic chips
* Maximally flexible

Cons:
* Puts the onus of ensuring that the chip inputs have been provided on the user,
  although with the way that the evaluator currently works a runtime error will
  be bubbled up to the user as soon as they try to run a machine with
  uninstantiated inputs

### Create a new cyclic chip abstraction which verifies inputs at compile time

We could create a cyclic chip abstraction which ensures at compile time that we
do not leave inputs uninitialised:

```rust
let (nand, testchip) = create_subchip(
    alloc,
    |(testchip,)| NandInputs {
        in1: in1.into(),
        in2: testchip.get_out(alloc).out.into(),
    },
    |(nand,)| TestchipInputs {
        in1: in2.into(),
        in2: nand.into(),
    },
);
```

Pros:
* Preserves compile time checking

Cons:
* Harder to implement
* Will probably need a separate implementation depending on how many chips we
  want in the cycle
* Makes the emulator code harder to verify

### Use a macro somehow

## Decision

I have gone with a combination of the first two solutions.

First, I've added an API to allow the creating of chip outputs detached
from the graph. Using this API directly risks runtime failures if the user is
not being careful.

Secondly, I've added a method which provides a safe abstraction for using the
partial chip API such that it will never result in a runtime failure.

The second step was quite easy once the first had been achieved.
