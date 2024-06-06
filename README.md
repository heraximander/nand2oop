# nand2oop

This project seeks to provide a Hardware Description Language (HDL) useful for developing hypothetical chipsets. It does not seek to emulate electrical dynamics, only logical operation.

It was used for implementing the exercises from _Elements of Computing Systems_ by Noam Nisan and Shimon Schocken.

If you're confused about the design decisions in this project, [my ADRs](docs/adr/) may shine some light.

## Design goals

This project is intended to be used as a Rust library rather than in an interactive manner. It is designed to use the Rust compiler to achieve static checking of gate connections.