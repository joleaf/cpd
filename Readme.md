[![Crates.io](https://img.shields.io/crates/v/cpd.svg)](https://crates.io/crates/cpd)
[![Downloads](https://img.shields.io/crates/d/cpd.svg)](https://crates.io/crates/cpd)
[![Documentation](https://docs.rs/cpd/badge.svg)](https://docs.rs/cpd)

# CPD

**Collaboration Pattern Discovery (CPD)** is a subgraph mining algorithm that searches for patterns in a graph database.
The algorithm does not only look for exact matches; it can also explore a relaxed search space.
Furthermore, based on the defined vertex_types, the algorithm identifies context-aware patterns.

## Usage

### Install

```shell
cargo install cpd
```

Or download the binary from the [latest release](https://github.com/joleaf/cpd/releases/latest).

### Input graph database

Provide a plain text file with the graph database:

- t-line: Begin of a new graph
    - Format `t # i` 
        - `i` (int) index of the graph
        - (The result file contains 2 additional values: `* {exact_frequency} / {relaxed_frequency}` )
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
see [graph.txt](./test_data/graphs.txt)
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
Example [graph database](./test_data/graphs.txt):
```shell
cpd \
    --input test_data/graphs.txt \
    --activity-vertex-type 3 \
    --object-vertex-types 1 \
    --support-exact 2 \
    --support-relaxed 5 \
    --graph-matching cosine
    --relaxed-threshold 0.6 \
    --min-vertices 3 \
    --max-vertices 3 \
    --alpha 0.5 \
    --output out.txt
```

Example [small graph database](./test_data/graphs_small.txt) (only 5 graphs):
```shell
cpd \
    --input test_data/graphs_small.txt \
    --activity-vertex-type 1 \
    --object-vertex-types 6 7 \
    --support-exact 2 \
    --support-relaxed 3 \
    --graph-matching cosine
    --relaxed-threshold 0.8 \
    --min-vertices 3 \
    --max-vertices 3 \
    --alpha 0.5 \
    --output out_small.txt
```
The parameter `--activity-vertex-type` specifies which vertex type is treated as an activity node; CPD will only generate pattern candidates where these activity vertices form a fully connected subgraph.
The parameter `--object-vertex-types` defines which vertex types represent context nodes, meaning they provide additional structural or semantic information that surrounds the activity pattern.
Together, these settings ensure that discovered patterns always contain a cohesive activity core enriched with contextual object information.

Get help:
```shell
cpd --help
```

```shell
 A tool to search for context-aware and relaxed frequent subgraphs in a graph database

 Usage: cpd [OPTIONS] --input <INPUT>

 Options:
   -i, --input <INPUT>
           Input file with the graph database
   -o, --output <OUTPUT>
           Output file for the resulting subgraphs, if "sdtout", the resulting patterns will be printed t o the console after processing finished with ###### [default: stdout]
       --support-exact <SUPPORT_EXACT>
           Exact support [default: 2]
       --support-relaxed <SUPPORT_RELAXED>
           Relaxed support [default: 2]
       --graph-matching <GRAPH_MATCHING>
           Graph matching: - "cosine" (node and edge vector similarity, uses the alpha parameter), - "ged " (approx. graph edit distance) - "vf2" (only exact matches) [default: cosine]
       --relaxed-threshold <RELAXED_THRESHOLD>
           Relaxed threshold, 0.0 - 1.0 for graph matching "cosine", and >= 0 for graph matching "ged" [d efault: 0.8]
       --activity-vertex-type <ACTIVITY_VERTEX_TYPE>
           Activity vertex type [default: 0]
       --object-vertex-types [<OBJECT_VERTEX_TYPES>...]
           Object vertex types
       --min-vertices <MIN_VERTICES>
           Minimum number of main vertices [default: 4]
       --max-vertices <MAX_VERTICES>
           Maximum number of the main vertices [default: 5]
       --alpha <ALPHA>
           The alpha value between 0.0 and 1.0 defines the weight importance of the vertex and edge vecto rs: if 1.0, the edges are ignored; if 0.0, the vertices are ignored [default: 0.5]
       --silence
           Supress debug statements
   -h, --help
           Print help
   -V, --version
           Print version

```

#### Implemented Graph Matcher
For the parameter `--graph-matching`, the following options are valid:
- `cosine`: The cosine similarity is calculated based on the vertex and edge vectors. The `--alpha` parameter is used to weight the impact of both similarities. If the similarity is 1.0, the graphs may be identical; if the value is 0.0, the graphs are completely different. The `--relaxed-threshold` defines whether two graphs are similar enough to be treated as a relaxed match. However, a similarity of 1.0 does not mean that the graphs are structurally identical; in such cases, the `vf2` algorithm checks for exact matches.
- `ged`: An approximate implementation of the graph edit distance using the [Hungarian](https://en.wikipedia.org/wiki/Hungarian_algorithm) algorithm. A value of 0 means no change operations are needed to transform one graph into the other. A GED value of 2 means two change operations (e.g., insertion, deletion, or substitution of vertices or edges) are needed to transform one graph into another. Since the result is an approximation, the `vf2` algorithm is also used to check for exact matches if GED returns 0.
- `vf2`: The [VF2](https://doi.org/10.1016/j.dam.2018.02.018) algorithm checks for exact matches using graph isomorphism.

