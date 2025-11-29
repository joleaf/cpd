# gSpan - *RUST Implementation*

**gSpan** is an algorithm for finding frequent (sub-)graphs in a graph database.
This [Rust](https://www.rust-lang.org/) implementation is based on
the [gSpan.java](https://github.com/TonyZZX/gSpan.Java) implementation by TonyZZX.
The purpose of this reimplementation is the very fast performance of Rust to further improve the performance.
For details on the gSpan algorithm, we refer to the original work by Xifeng
Yan: [gSpan](https://sites.cs.ucsb.edu/~xyan/software/gSpan.htm).

# WARNING

**THIS PROJECT IS WIP!**

## Usage

### Install

Download the latest [Release]().

## Input graph database

Provide a plain text file with the graph database:

- t-line: Begin of a new graph
    - Format `t # i` -> `i`
- v-line: Definition of a vertex
    - Format `v i l` -> `i` (int): index of the vertex inside the graph; `l` (int): label of the vertex
- e-line: Definition of an edge
    - Format `e v1 v2 l`-> `v1` (int): index of the from-vertex of the graph; `v2` (int): index of the to-vertex of the
      graph; `l` (int): label of the edge

Example:

```
t # 0
v 0 1
v 1 2
v 2 3
v 3 4
v 4 5
e 0 1 1
e 0 2 1
e 1 3 1
e 2 3 1
e 3 4 1
t # 1
v 0 1
v 1 2
v 2 3
v 3 4
v 4 5
e 0 1 1
e 0 2 1
e 1 3 1
e 2 3 1
e 3 4 1
```

or see [test](test) for a large graph database.

### Run gSpan

```shell
./gspan --input test --support 100 --min-vertices 1 --max-vertices 10 --directed
```

Get help:
```shell
./gspan --help
```

```
Usage: gspan [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>                Input file with the graph database
  -o, --output <OUTPUT>              Output file for the resulting subgraphs [default: out.txt]
  -s, --support <SUPPORT>            Min support [default: 2]
      --min-vertices <MIN_VERTICES>  Minimum number of vertices [default: 1]
      --max-vertices <MAX_VERTICES>  Maximum number of vertices [default: 10]
  -d, --directed                     The graphs are directed
  -h, --help                         Print help
  -V, --version                      Print version      
```

## Performance tests

tba

## Dev & Build

Install [rustup](https://rustup.rs/) (cargo) and run:

```shell
RUSTFLAGS="-C target-cpu=native" cargo build --release --all-features && cp target/release/gspan .
```