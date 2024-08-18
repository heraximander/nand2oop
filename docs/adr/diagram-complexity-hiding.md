# Complexity Hiding for Chip Diagrams

## Background

Currently chips can be compiled to mermaid, a graph markup language. This provides
a graphical representation of all NAND gates in a chip, aiding
designing and debugging chips.

As the chips I define are getting more and more complex the generated chip
diagrams are becoming less legible and useful as they have too much information
on them. Often I am more interested in the high-level composition of a chip and
not in the gates inside the primitive chips which make up them.

## Problem

How can I make the diagrams less cluttered and more useful at a high level?

## Solutions

### Keep the diagrams as is

Pros:
* No extra work

Cons:
* Doesn't solve the problem
* Slow to render large chips

### Provide an interactive diagram experience

I could build an interactive experience whereby clicking on a chip node
will reveal its implementation. We should also allow for reverting back
to the previous view to allow the user to "zoom out" from implementation
details.

Pros:
* Flexible, gives the right level of abstraction
* We're not wasting render time on large diagrams of implementations the
  user doesn't need

Cons:
* Takes the most effort, as we now have to create a UI
* Mermaid.js won't keep the subgraphs in the same position as they're
  expanded. This means the UX is a bit janky between clicks as node jumps
  around.

### Extend the diagramming API to include a complexity level parameter

I could add to the diagramming API a parameter which sets the depth of
rendering the chip implementations.

Pros:
* Doesn't require creating a UI
* Allows hiding a lot of complexity

Cons:
* Doesn't allow different nodes to have different level of expansion.
* Changing the abstraction level requires a compile

### Only show the top level chip implementations

We could fix the chip abstraction level to only show the top-level chips.

Pros:
* Simplifies the existing code a lot
* Most of the time, this is all the information we need

Cons:
* Doesn't show any of the nested complexity of a chip, although the user
  could easily diagram individual chips with separate calls to the diagramming
  API

## Decision

I have decided to go with the interactive solution. This is mostly because
it adds powerful functionality and won't be too hard to implement if I continue
to use mermaid.