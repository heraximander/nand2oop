# Speed up machine execution

## Type

Feature

## Status

Open

## Description

Although some care was taken when defining and implementing the hardware
description language to keep it performant, no optimisation has taken place
and I will likely reach a point where it isn't running fast enough to emulate
an entire machine. This issue at the moment is more speculative and is more a
place to dump optimisation ideas as I encounter them.

Possible optimisations:
1. See if we can avoid checking each input/output object as we traverse the
   execution graph, and instead only load the NAND gates
1. Compile the execution graph down to a set of binary comparisons instead of
   traversing a graph at runtime. Might be tricky to do with cycles.
1. Compile simple chips down to a truth table. A simple chip could be defined
   as a non-cyclic chip with N or fewer inputs. Is there any case in which you
   can algorithmically discover the truth table size for cyclic chips?
