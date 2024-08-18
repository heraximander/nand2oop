# Modelling Sequential Components in HDL code

## Background

The HDL was initially designed with only combinatorial circuits, not sequential
circuits, in mind. I am now getting to the stage in the textbook where I need to
model sequential components. The defining feature of sequential components is
that they not only depend on their current inputs, but also past inputs. In
hardware they are usually represented as mutually dependent gates, that is the
output of one gate is the input to another which is also dependent on the first
gate. This cannot currently be represented in HDL.

## Problem

How can I represent sequential components in HDL?

## Solutions

### Introduce a new axiomatic component, the flipflop

The simplest solution would be to introduce a new fundamental component which
models the only sequential component I need: the Data FlipFlop (DFF).

Pros:

* Easy to implement
* Performant, as it reduces the need to model its constituent gates

Cons:

* Complicates the HDL domain
* Does not allow for modelling other types of sequential circuits
* Does not let me demonstrate my understanding of how flipflops are actually
  implemented

### Allow for cycles in the HDL graph

I could instead allow for cycles to be modelled in HDL

Pros:

* Lets me demonstrate my understanding of sequential component implementations
* Lets me toy around with different sequential component implementations
* Keeps the number of fundamental components low, and therefore keeps HDL domain
  simple

Cons:

* Harder to implement
* Harder to optimise
* Making a statically verifiable interface to a cyclic graph is hard to do, and
  a design objective to HDL is to allow static verification of inputs to make
  sure a component has enough inputs
* Hard to model accurately behaviour such as unstable states, which might let
  bugs passed unnoticed. For example, if an SR latch has inputs of 1,1 it should
  have an unstable output

## Decision

As this is a learning project, I decided to try implementing cycles in HDL so I
get more experience in writing fundamental logic.
