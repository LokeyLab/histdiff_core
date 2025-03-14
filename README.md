# histdiff_core

This is the main histdiff core library. This is will be used as a submodule for other histdiff related projects. If you want to directly use HistDiff please use HistDiff_standalone

## by Derfel Terciano written in Rust

### Purpose

The main purpose of this is to create a common HistDiff package for other programs to use.
Additionally, I want to attempt to refactor HistDiff to have the lowest memory profile as
possible.

_Note: Logging is used to display algorithm progress._

### Datasets to use for development:

use: TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv

## Development and re-implementation complete

### Usage/Documentation:

Run `cargo doc --open` to see the full documentation of the functions and structs

### NOTES:

- Vehicle specification must not contain any leading zeroes i.e.
  - "F08" will not work but "F8" will work.
  - All cell data inputs must NOT have **_ANY LEADING ZEROES in name._**
