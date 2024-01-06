# A-Puzzle-A-Day

[DragonFjord's A-Puzzle-A-Day](https://www.dragonfjord.com/product/a-puzzle-a-day/)
tasks you with placing eight pieces within a calendar frame
to reveal the current date. There are roughly 60 thousand ways
the pieces can fit in the frame, and of those arrangements
over 24 thousand are valid solutions.
That is an average of 67 solutions per date, and yet some days
solving the puzzle can seem utterly and hopelessly impossible.

This piece of software can serve as a holy light for those
despondent, ego-shattering days, allowing you to generate
and browse all solutions for all days at your leisure.

## Installation

Currently, this software can only be installed from source, using [Cargo](https://doc.rust-lang.org/stable/cargo/):
```
$ cargo install --git https://github.com/mrbjarksen/a-puzzle-a-day
```
This will place the exectuable `a-puzzle-a-day` into `$HOME/.cargo/bin`
(by default), which should be added to `$PATH`.

## Usage

There are three main modes of use: `generate`, `browse`, and `random`.

### Generate
```
$ a-puzzle-a-day generate
```
This will generate all solutions and write them to the file `solutions.apad`,
or the file specified by the option `-f` or `--file` if found
(this file is identical to the solutions file found in this repository).

Solutions are found using brute-force, each piece placed on each square in parallel.
Care has been made in minimizing the amount of work needed, but the generation will
take at least a few seconds and a few dozen threads.

### Random
```
$ a-puzzle-a-day random
```
This will display a random solution for the current date, or the date specified by
the option `-d` or `--date`.

### Browse
```
$ a-puzzle-a-day browse
```
Open a TUI showing all solutions. This mode is still a work in progress.
