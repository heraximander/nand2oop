# Refactor`UserInput` struct usage

## Type

Tech debt

## Status

Open

## Description

The `UserInput` struct was initially named because it was only used for settable
value user inputs. With some new chips such as `incrementor` I am using it for
fixing a subchip input to a specific value. In this context the name `UserInput`
seems a bit odd.

I should either refactor chips such as `incrementor` to use a different `Input`
struct or rename `UserInput` to something else.
