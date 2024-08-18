# Simplify visualisation of array inputs

## Type

Feature

## Status

Open

## Description

Some chips have arrays of inputs, for example the 16 bit adder chips. These are
currently visualised with each element of the array having its own node. This
fills up the graphs with too many nodes, making the diagram much less legible.
We should simplify this representation. Some possible ways we could do this are:

1. Add names to inputs as per [this issue](./chip-diagram-names-inputs-outputs.md)
1. Group inputs and outputs in to subgraphs to force better rendering
1. Group inputs if they are an array
1. Combine array inputs in to a single graph node. We might want to allow click to
   expand if the users need to know which input goes where, which I can only
   imagine you'd want to do if you wanted to see which way a `mult` chip is
   aggregated (ie left-to-right or right-to-left)
   1. Alternatively we could group all outputs coming from a single output source
      and are going to a single input source, and group all inputs coming from a
      single output source and are going to a single input source. This is
      probably midly preferable, but would be a pain to implement given the
      current chip graph structure