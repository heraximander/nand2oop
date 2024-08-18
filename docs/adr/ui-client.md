# Client to render interactive UI

## Background

As per [the diagram complexity ADR](./diagram-complexity-hiding.md) we require
an interactive user interface. Currently the mermaid diagrams are output as
mermaid markup and it is up to the user to render the markup themselves. This
is hard to do if we require redrawing diagrams in response to user input.

## Problem

How should we present the interactive user interface?

## Solutions

### Use a webserver package

We could use a webserver and web frontend to render the diagrams and respond to
user inputs.

Pros:
* Provides an out-of-the-box interactive solution
* I get to learn how to use a Rust web framework

Cons:
* Ties the UI package to a particular implementation of rendering mermaid graphs
* Requires a browser to view the user interface
* Adds more dependencies to the project, probably including a runtime eg Tokio

### Write my own webserver

Instead of using a common webserver package, we could write the webserver itself.

Pros:
* Reduces number of dependencies
* Don't need to introduce a runtime to the project
* Reduces new technologies I have to learn

Cons:
* Might be more effort than using a webserver package, although it might actually
  take me less time because webserver libraries are normally relatively complex
  and take time to learn
* I probably will end up using a single-threaded, blocking webserver which won't
  scale well

### Use a native client

I could use a native client to render the graphs

Pros:
* Probably a better UX than having to load a browser, especially as the window
  can close when the program is terminated
* Maybe less resource overhead
* Might be an even better UX if I have to add more complex UI functionality later
* I get to learn how to make native GUI applications

Cons:
* Mermaid support is best for rendering to HTML, so I might have to render to
  an HTML frame anyway inside the native window
* Will probably take quite a bit of time to implement

### Let the user choose how to render the UI

We could let the user manage rendering the UI, letting the user pipe that to their
favourite rendering tool as they currently can.

Pros:
* Lets the user use their own GUI in the way they want

Cons:
* I can't imagine the interactive part being particularly fluid. It definitely won't
  be click-based and would probably require the user to repeatedly run CLI commands
  when changing abstraction level.
* If I'm the only one using the project, I can hardcode the UI that I want in the
  application

## Decision

I decided to go with an interactive webserver as it provides the best UX at the
lowest effort. I will write my own webserver as I believe it might be faster than
using another package for this simple usecase.

## Post-op

It was indeed easy to write my own webserver. That said, if UI needs get more
complex I may have to revisit that decision.