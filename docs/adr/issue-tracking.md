# Issue Tracking

## Background

Most software projects I've worked on use an issue tracker to track new
features, tech debt and bugs. I am currently tracking issues in a comment
in [the `project` package](../../project/src/main.rs). As the number of
features I'm thinking of adding and tech debt starts to grow it is getting
unwiedly listing them in a rust comment, and makes me not want to list new
issues that I might not work on immediately.

## Problem

I want to use an issue tracker

## Solutions

### Use current solution 

Pros:
* No change from I'm currently doing

Cons:
* Makes the project file a bit unwieldy

### Use GitHub issues

Pros:
* Lots of features
* Makes organising issues easy
* Can have a discussion on issues

Cons:
* Can't use when I don't have internet

### Use markdown files in an `issues/` directory

Pros:
* Simple
* Clean: all issues are in a subdirectory which I can exclude from history
  if I only want to view code changes
* Ties issues closely to the commits which resolve them

Cons:
* Not industry standard, others won't expect it to work in this way
* Not as good organisation/searchable as a dedicated application
* Easy to create issues which don't conform to a template. I could create a
  linter for that, but it's massively overcomplicating things

## Decision

I'm going to go with using markdown files in an `issues/` directory because
internet connectivity is a real problem for me at the moment.

I suspect a few of the downsides aren't as problematic as they appear.
Searchability should be okay via standard unix commandline tools, and as I'm the
only one contributing to this project I can stick to a convention for writing
issues.

I also find that any conversation around an issue shouldn't really be on the
issue itself, as it makes reading the current state of an issue really difficult.

For now I'll implement the issues as markdown documents but TOML might make more
sense for a future change.

Expect a post-op!
