# Document the HDL evaluation strategy

## Type

Feature

## Status

Open

## Description

When I built the initial cut of the emulator I decided to use a specific
evaluation strategy, recursing from outputs to inputs and caching intermediate
results on the nodes. I should document why I did this, especially as it turns
out it is _very_ handy for evaluating graph cycles.