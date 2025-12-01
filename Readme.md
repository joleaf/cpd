# CPD

**Collaboration Pattern Discovery (CPD)* is a subgraph mining algorithm that searches for patterns in a graph database.
The algorithm does not only look for exact matches; it can also explore a relaxed search space.
Furthermore, based on the defined vertex_types, the algorithm identifies context-aware patterns.

## WARNING

**THIS PROJECT IS WIP!**

## Usage

### Install

```shell
cargo install cpd
```

### Input graph database

Provide a plain text file with the graph database:

- t-line: Begin of a new graph
    - Format `t # i` 
        - `i` (int) index of the graph
        - (The result file contains 2 additional values: `* {exact_frequence} / {relaxed_frequency}` )
- v-line: Definition of a vertex
    - Format `v i l t` 
        - `i` (int): index of the vertex inside the graph
        - `l` (int): label of the vertex
        - `t` (int): type of the vertex
- e-line: Definition of an edge
    - Format `e v1 v2 l`
        - `v1` (int): index of the from-vertex of the graph
        - `v2` (int): index of the to-vertex of the graph 
        - `l` (int): label of the edge


#### Example
see [graph.txt](./graphs.txt)
In this example, the vertex type 1 is the "activity" and the vertex type 3 defines the context of the activity vertices.
```
t # 0
v 0 0 1
v 1 2 3
v 2 4 1
v 3 2 3
v 4 5 1
v 5 2 3
v 6 6 3
e 1 0 1
e 3 2 1
e 1 3 1
e 5 4 1
e 3 5 1
e 6 2 1
e 5 6 1
e 6 6 1
t # 1
v 0 7 1
v 1 2 3
v 2 8 1
v 3 9 3
v 4 9 3
v 5 10 1
v 6 9 3
v 7 11 1
v 8 2 3
v 9 12 1
v 10 2 3
v 11 13 1
v 12 2 3
e 1 0 1
e 3 2 1
e 1 3 1
e 4 0 1
e 3 4 1
e 4 3 1
e 6 5 1
e 3 6 1
e 8 7 1
e 6 8 1
e 10 9 1
e 8 10 1
e 12 11 1
e 10 12 1
```

### Run cpd
```shell
cpd \
    --input graphs.txt \
    --activity-vertex-type 3 \
    --object-vertex-types 1 \
    --support-exact 2 \
    --support-relaxed 5 \
    --relaxed-threshold 0.5 \
    --min-vertices 3 \
    --max-vertices 3 \
    --output out.txt
```

The parameter `--activity-vertex-type` specifies which vertex type is treated as an activity node; CPD will only generate pattern candidates where these activity vertices form a fully connected subgraph.
The parameter `--object-vertex-types` defines which vertex types represent context nodes, meaning they provide additional structural or semantic information that surrounds the activity pattern.
Together, these settings ensure that discovered patterns always contain a cohesive activity core enriched with contextual object information.

Get help:
```shell
cpd --help
```

```shell
A tool to search frequent subgraphs in a graph database

Usage: cpd [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
          Input file with the graph database
  -o, --output <OUTPUT>
          Output file for the resulting subgraphs, if "sdtout", the resulting patterns will be printed to the console after processing finished with ###### [default: stdout]
      --support-exact <SUPPORT_EXACT>
          Min exact support [default: 2]
      --support-relaxed <SUPPORT_RELAXED>
          Min relaxed support [default: 2]
      --relaxed-threshold <RELAXED_THRESHOLD>
          Relaxed threshold [default: 0]
      --activity-vertex-type <ACTIVITY_VERTEX_TYPE>
          Activity vertex type [default: 0]
      --object-vertex-types [<OBJECT_VERTEX_TYPES>...]
          Object vertex types
      --min-vertices <MIN_VERTICES>
          Minimum number of main vertices [default: 4]
      --max-vertices <MAX_VERTICES>
          Maximum number of the main vertices [default: 5]
      --silence
          Supress debug statements
  -h, --help
          Print help
  -V, --version
          Print version  
```

## Performance tests

tba

## Dev & Build

Install [rustup](https://rustup.rs/) (cargo) and run:

```shell
RUSTFLAGS="-C target-cpu=native" cargo build --release --all-features && cp target/release/cpd .
```
