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

## Usage

There are three main modes of use: `browse`, `random`, and `generate`.

### Browse

![image](https://github.com/mrbjarksen/a-puzzle-a-day/assets/62466569/986c1024-3a14-481c-bad9-dea56a74ec77)

```
$ a-puzzle-a-day browse [-f/--file <FILE>] [-d/--date <DATE>]
```

Open a TUI showing all solutions. The UI is split into two panes: the date pane (left) and the solution pane (right).
To navigate, use the arrow buttons or hjkl. To switch panes, press Tab or Enter. To quit, press Escape or q.

Alternatively, there is full mouse support, including scroll.

Note that colors were chosen with a dark terminal theme in mind. There is currently no way to change colorschemes.

### Random

```
$ a-puzzle-a-day random [-f/--file <FILE>] [-d/--date <DATE>]
```

This will display a random solution for the current date, or the date specified by
the option `-d` or `--date`.

### Generate

![generate](https://github.com/mrbjarksen/a-puzzle-a-day/assets/62466569/03eb50bb-c795-42b9-9f35-5eebe4e05776)

```
$ a-puzzle-a-day generate [-f/--file <FILE>]
```

This will generate all solutions and write them to the file `solutions.apad`,
or the file specified by the option `-f` or `--file`
(this file is identical to the solutions file found in this repository).
The file created uses a custom-built binary file format, named [APAD](docs/APAD.md).

Solutions are found using brute-force, each piece placed on each square in parallel.
Care has been made in minimizing the amount of work needed, but the generation will
take at least a few seconds and a few dozen threads.

**Note**: this is generally unnecessary, as the solutions are included
in the binary.

## Installation

Currently, this software can only be installed from source, using [Cargo](https://doc.rust-lang.org/stable/cargo/):
```
$ cargo install --git https://github.com/mrbjarksen/a-puzzle-a-day
```
This will place the exectuable `a-puzzle-a-day` into `$HOME/.cargo/bin`
(by default), which should be added to `$PATH`.
