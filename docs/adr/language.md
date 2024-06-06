# HDL Language

## Background

I want to study the textbook _Elements of Computing Systems_ by Noam Nisan and Shimon Schocken. A big part of this textbook is the exercises that come with, which make you build a computer up from NAND gates to a full OOP programming language running on top of an operating system. The authors provide a Hardware Description Language (HDL) and an emulator to run the chips on.

I have also recently picked up Rust as a programming language I want to learn.

## Problem

Which HDL language and simulator should I use for this project?

## Solutions

### The one provided with the book

I haven't actually used the emulator so I don't know how good it is for sure.

Pros:
* Will get to working on the exercises quickly - I only need to work out how to get the program to run
* Works with the book's provided test cases
* Proved correct for the purposes of the book
* Probably more performant than something I'll write myself
* Probably has better debugging features than something I'll write from scratch
* Is written by people with a lot more knowledge of HDL tools than I, so it will probably be better designed

Cons:
* The static validation might not be very good
* Don't get a chance to learn how to write an emulator

### Write my own HDL + parser + emulator

Pros:
* Get to learn how to write an emulator
* I can write the best HDL for my needs, unconstrained by what was provided by the textbook or the DSL possible in the textbook

Cons:
* Need to write a parser, which may take longer than creating a DSL
* All the pros of using the HDL provided by the authors are cons for this

### Create an HDL DSL and emulator in a high-level programming language

Instead of writing a parser, I could use a high-level language to create a new DSL for this project.

Pros:
* Don't have to write a parser
* Can use existing dev tools for the language, for example linting and language server
* May be easier to create a more performant version emulator if I use a compiled language

Cons:
* I am constrained by the flexibility of the language I choose
* Some HDL constructs may not be able to be checked at compile time, which negates some of the advantages of this approach

## Decision

I have decided to create an HDL DSL and emulator in Rust. This is mostly because I place a lot of importance on learning Rust and this seems like a good project to do it on - and kill two birds.

I believe Rust itself is a good language to do this in because:
* It is compiled and is known to be easy to create performant programs
* It uses a lot of zero-overhead abstractions which should also help performance
* It has a robust type system and macro system which gives me a lot of flexbility in creating the DSL
* The robust type system means that most HDL constraints should be able to be checked at compile time, not run time (for example array bounds checking)

## Post-Op

I've made extensive use of tagged unions in the DSL to represent things such as the set of all possible inputs to a gate (for example, a `ChipInput` or an output from another chip `ChipOutput`). This avoids any need for dynamic dispatch, which I want to avoid for compile-time correctness and performance reasons (intuitive, not measured). This adds a lot of boilerplate for the user as they need to specify the variant of the enum. Ideally I'd have a sum type where the user does not need to specify the variant but Rust does not support this, outside of its `Union` type which requires `unsafe{}` to handle. 