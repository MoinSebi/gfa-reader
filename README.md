# gfa-reader - Reading GFA files

Able to work with version  1.0, 1.1, 1.2 and 2.0 in a single structures.   
**GFA format specification:**
- v1: https://gfa-spec.github.io/GFA-spec/GFA1.html
- v2 https://gfa-spec.github.io/GFA-spec/GFA2.html


### Installation
```
gfa-readder = { git = "https://github.com/MoinSebi/gfa-reader", branch = "main" }
```
**OR** 
```
gfa-reader = "0.1.4"
```
### General information:
gfa-reader has two main structs: Gfa for versions 1.0, 1.1, 1.2 and 2.0 and NCGfa for version 1.2 or lower.   
**Gfa** represents the basic implementation for all versions and node ids. As stated in the specification, node ids can be numeric or alphanumeric, therefore represented as a String in our implemenation. This can lead to increased memory.  
**NCGfa (NumericCompact)** is a compact representation of the graph with 
numeric and compacted (starting at 1) node ids. Can be used for variation/genome graphs from [pggb](https://github.com/pangenome/pggb) or [minigraph-cactus](https://github.com/ComparativeGenomicsToolkit/cactus/blob/master/doc/pangenome.md).

#### Optional fields
Several GFA entries have optional fields. Most of the time, these fields are 
not needed for the basic graph structure. Therefore, they can manually read, if needed or left out.
This option will be set once for all entries, which either parse or don't parse the optional information.

#### Edges
In several specific cases, edges are not needed since the graph structure can be represented with the path information. The collection of edges are represented as `Option<edge>`, giving the possibiliy to not populate the structure at all of not needed. 

```rust
use gfa_reader::{NCGfa, OptElem};

// No edges and no optional fields
let mut graph: NCGfa<()> = NCGfa::new();
graph.parse_gfa_file("data/size5.gfa", false);

// Edges and no optional fields
let mut graph: NCGfa<()> = NCGfa::new();
graph.parse_gfa_file("data/size5.gfa", true);


// Edges and optional fields
let mut graph: NCGfa<Vec<OptElem>> = NCGfa::new();
graph.parse_gfa_file("data/size5.gfa", true);
```
## PanSN
Pan-SN spec is a specification for storing variation graphs in a GFA format. It is strongly supported by gfa-reader with a pansn struct. It allows you to utilize genome, haplotype or path level collections, dependent on the use case.  
If the data is not in pansn format, each path will represent its own genome and haplotypes. Take care that you have the same delimiter twice. 

```rust
use gfa_reader::{Gfa, Pansn, Path};
let mut graph: Gfa<()> = Gfa::new();
graph.parse_gfa_file("data/size5.gfa", false);
let pansn: Pansn<Path> = Pansn::from_graph(&graph.paths, " ");

```

## Walks
Walks are "alternative representation" of paths in the graph. We can convert walks to path using PanSN-spec. The start and and end of the walk are concatenated at the end of the path name. We add a non-existing Overlap "*" as the for the path, since this information is not given in the walk specification. 


```rust 

```rust

### Additional information
We recommend using NCGfa in every scenario since there are two main advantages:
1. You can convert a Gfa to a NCGfa and get a Vector data reflects new_id (usize) -> old_id (string)
2. For node lookup you don't need get additinal HashMaps or iterate over the whole vector, you can just take the node id minus 1 look it up directly in the vector. 

### Tips
- For any graph-related output which is based on the features of the graph, don't forget to re-convert node id in order they fit to the input graph structure.
- Convert the graph to numeric and compact node ids before parsing. This saves time for parsing and makes computation faster.


