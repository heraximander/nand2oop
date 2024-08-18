# HDL Structured Chip Inputs and Outputs

## Background

The HDL DSL currently requires the user to provide inputs to a chip as an array, such as:

```rust
fn adder16<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 32]) -> [ChipOutputType<'a>; 16] {
    ...
}
```

The DSL gives no indication as to how each element in the array changes the output. In the
example above, the first 16 entries of `input` refer to the first number to be added, the
next 16 are for the second number.

The DSL also provides outputs in array form, such as:

```rust
fn fulladder<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 3]) -> [ChipOutputType<'a>; 2] {
    ...
}
```

This also gives the user no indication how the outputs change in relation to the inputs. In
this example the former entry in the output array is the carry bit; the latter is the sum bit.

This will get even harder for the full ALU chip specification, as it has many diverse inputs
and outputs.

Note that this input and output structure is also used by the `Machine` construct for processing
inputs, as well as in the chip constructor:

```rust
pub fn process(&mut self, input_vals: [bool; NINPUT]) -> [bool; NOUT] {
    ...
}
```

## Problem

The input/output format of the current chip DSL does not assign meaning to each input and output.
This makes complex chip specifications hard to understand.

## Solutions

### Input

#### Keep the current input structure

I could keep the current input structure of an input array. In this case it would make sense to
create a new convention for labelling inputs, perhaps via a comment.

Pros:
* No effort required
* Current representation is compact
* Current chip designs don't have complex schemas so we would not get the full benefit of
  labelling inputs yet.

Cons:
* Makes it easy to reference the wrong inputs 

#### Allow multiple arguments

I could add the ability to have multiple arguments to the chip function, for example like:

```rust
fn adder16<'a>(alloc: &'a Bump, num1: [&'a ChipInput<'a>; 16], num2: [&'a ChipInput<'a>; 16]) -> [ChipOutputType<'a>; 16] {
    ...
}
```

Pros:
* Provides good indication of what the arguments are
* Relatively easy to implement
* Relatively compact representation

Cons:
* This doesn't provide a solution of how to manage inputs in the `Machine::process()` method.
  I could revisit whether the `Machine` abstraction is necessary and whether instead I could
  `::process()` on the chip directly, but as this also involves macro generation it is not
  clear if it is easier to implement than the alternatives below.
* It is unclear how `Machine` will construct the inputs to `::new()`

#### Pass arguments via an input struct

Instead of taking labelled arguments directly, the chip functions could take an input struct,
such as below:

```rust
struct Inputs<T> {
    num1: [T; 16],
    num2: [T; 16]
}
fn adder16<'a>(alloc: &'a Bump, inputs: Inputs<&'a ChipInput<'a>>) -> [ChipOutputType<'a>; 16] {
    ...
}
```

Pros:
* The input struct can be reused for `Machine::process()`, so both the chip constructor and
  chip processing can use the meaningfully labelled inputs

Cons:
* The inputs are defined separately to the chip definition function
* There is a bit more boilerplate in defining the struct and constructing the struct before passing
  it in to the `::new()` function
* It is unclear how `Machine` will construct the inputs to `::new()`. We could use a macro
  to convert an array of `T` to a struct of `Input<T>`, but this would add extra macro complexity.
  Macros have proven to be the most time-consuming part of the project to program and the most
  fragile. 

#### Provide an array schema

The chip definition function could accept an array of input, but a schema provides meaning to the
different array elements, for example:

```rust
const MUX_IN1: usize = 0;
const MUX_IN2: usize = 1;
const MUX_SEL: usize = 2;
fn mux<'a>(alloc: &'a Bump, num1: [&'a ChipInput<'a>; 3]) -> [ChipOutputType<'a>; 1] {
    ...
}
...
let mux_in = [&'a ChipInput<'a>; 3]
mux_in[MUX_IN1] = in1;
mux_in[MUX_IN2] = in2;
mux_in[MUX_IN3] = in3;

let muxchip = Mux::new(alloc, mux_in);
```

Pros:
* Simple to implement (convention-based rather than adding new features)
* Provides schematic information

Cons:
* I'm not sure the schema constants would work that well with ranges, eg `const ADDER16_IN: Range<usize> = ..16;`
* Is a bit verbose
* Makes it easy for the user to forget to instantiate an array element

### Output

#### Keep the current output structure

I could keep the current output structure of an output array. In this case it would make sense to
create a new convention for labelling outputs, perhaps via a comment.

Pros:
* No effort required
* Current representation is compact
* Current chip designs don't have complex schemas so we would not get the full benefit of
  labelling outputs yet.

Cons:
* Makes it easy to reference the wrong outputs 

#### Create an output struct

The chip functions could return an output struct, such as below:

```rust
struct Outputs<T> {
    sum: T,
    carry: T
}
fn half_adder<'a>(alloc: &'a Bump, inputs: [&'a ChipInput<'a>;3]) -> Outputs<ChipOutputType<'a>> {
    ...
}
```

Pros:
* The output struct can be reused for `Machine::process()`, so both the chip constructor and
  chip processing can use the meaningfully labelled outputs

Cons:
* The outputs are defined separately to the chip definition function. That said, some of the output
  struct definitions can be reused (for example the half and full adders have the same output
  structure)
* There is a bit more boilerplate in defining the struct and constructing the struct before returning
  it from the chip definition function
* With the way `Machine` currently reads the results of a process cycle from its chip outputs, we
  will need to provide a function for converting from a slice to an output type, which will most
  probably need to be written using a `#[derive()]` macro.
  Macros have proven to be the most time-consuming part of the project to program and the most
  fragile. 

#### Provide an output schema

We could still return an array of output, but provide constants as a schema:

```rust
const MUX_SUM: usize = 0;
const MUX_CARRY: usize = 1;
fn half_adder<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 2] {
    ...
}
...
let out = HalfAdder::new(alloc, input);
let carry = out[MUX_CARRY];
```

Pros:
* Simple to implement (convention-based rather than adding new features)
* Provides schematic information

Cons:
* Is a bit verbose as the schematic information isn't encoded in the return object
* Easy to forget to not define an output, or define two constants for the same output

## Decision

For the input I decided on a combination of input structs and named function arguments.
This will be implemented by the chip definition macro defining a struct based on the
chip definition function parameters. I rejected the array schema as it is too easy for the
user to forget to wire up an input; static checking was the point of writing this project
in Rust. I rejected the idea of not having an input struct as without it the `Machine`
abstraction would have a different interface to the chip. Finally, I decided to not keep
the code as-is because my attempts to define the ALU have been hard without some schematic
information.

For the output I have also chosen to use structs to be consistent between inputs and
outputs.

Both of these decisions have resulted in more boilerplate added to the DSL, which is
suboptimal but worth it.
