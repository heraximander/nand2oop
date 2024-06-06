# Representing graphs in the Rust DSL

## Background

Inherent in the HDL DSL is the ability to create graphs; a single chip or gate has other chips or gates connected to its inputs and its outputs. Rust famously makes it [difficult to represent graphs in code](https://aminb.gitbooks.io/rust-for-c/content/graphs/) because of the need to convince the compiler how long your node should last for (lifetimes). 

The design for the DSL is to have gates represented as `struct`s, with the constructors requiring references to other structs as inputs, for example (psuedocode):

```rust
let input1 = Input::new(false);
let input2 = Input::new(true);
let gate1 = Nand::new(&input1, &input2);
let gate2 = Nand::new(&gate1, &input1);
```

The idea is to have custom chips also represented as structs:

```rust
...
let chip = CustomChip1::new(&input1, &input2, &gate2);
```

If the gates inside a chip are created in the chip's constructor, which component owns those gates? The logical answer is the chip struct itself. However the chip also needs _references_ to those gates to feed in to the other gates stored inside that chip. This gets difficult to do in the constructor, because the memory locations of the owned gates will change as the `struct` is passed back to the caller.

## Problem

Which components own the gates inside custom chips and how do other gates reference those?

## Solutions

### Use `Rc`

I could put all gates behing `Rc` smart pointers. This means the ownership of gates is shared between all references, similar to how it would be in a garbage collected language.

Pros:
* Easy to implement
* Somewhat idiomatic
* The `#[chip]` macro doesn't need to do as much magic as if I tried the memory pinning option

Cons:
* Makes the code a bit more verbose with `Rc` types everywhere
* Adds runtime overhead when allocating and deallocating. That said, this only really happens while constructing the graph, which only happens once.
* _Somewhat_ leaking implementation details; I would prefer the user to be able to pass around `&` references without knowing the exact smart pointer I use. Probably a weaker point, but this just feels slightly icky to me 

### Try use memory pinning and unsafe references

I researched this a little, and it may be possible to partially-initialise the `Chip` struct with the gates as we create them, with pinned memory locations. We can then use normal `&` references to the pinned values, which can also be added to the return struct. I don't know if this is possible for sure and didn't research it extensively as there were enough cons to this approach already.

Pros:
* Avoids usage of extra `Rc` types visible to the user
* Avoids the requirement for a global arena, tying all lifetimes to it and the need to drill the arena down through constructors

Cons:
* May not be possible
* Would need to use a lot of `unsafe{}` even if it is possible
* Would need to choose between requiring the macro to abstract the pinning stuff and exposing that to the user. I can't think off the top of my head how abstract those details.

### Use an arena allocator

I could use an arena allocator to store the gates, and then all references have the same lifetime as the arena. I could have a single global arena or an arena per chip. The latter might be difficult if we wanted to create the arena, store it in the chip struct and store references to it in the same constructor. This is because the arena location will change as the chip struct is returned. There might be a simple solution to that, but I can't see it yet.

Pros:
* Simple
* Rust has many arena libraries
* All gates should have the same lifetime - we don't update the graph at runtime
* Would allow the user to use normal Rust `&` references

Cons:
* If we use a global arena (which is the likely outcome) we would need to drill that down through all chips, and will be visible to the  (thus a part of the API) - unless there's a simple solution I'm missing.
* It might be tempting to place things in the arena which don't need to be so long-lived, increasing the memory outlay. 

## Decision

Note that these are only the solutions I have thought of, and I have a limited knowledge of Rust.

I decided to use an Arena allocator. This was mostly for the aesthetic reason of avoiding an extra `Rc` type in the API. The hope is that eventually I'll think of a way to encapsulate the arena, avoiding having the arena allocator as a part of the API. It also avoids a lot of reasoning about lifetimes, at the expense of potentially storing things in the arena which don't need to be there. These are fairly weak reasons to prefer arenas over `Rc`, so it could have easily gone the other way. Experience will tell me whether this was a good choice.

## Post-Op

I couldn't abstract the creation of the arena behind `Machine::new()` as it would require a self-referential struct. This adds my choice of arena allocator, `bumpalo`, to my API.