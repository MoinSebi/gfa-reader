# gfa-reader - gfa reader

Library for reading gfa (v1) files. 

```
git clone https://github.com/MoinSebi/gfa-reader
```
### General information:
Both graph representations (Gfa and NCGFA) have the following composition:
``` test
- Node = Vec<Node>
- Edges = Vec<Edge>
- Path = Vec<Path>  
```

In addition, you can decide if you want to use and parse optional fields (in nodes and edges). To reduce runtime and memory you are also ignore edges and don't parse them at all. Since edges are also represented in the path information, this is sufficient for some use cases.
```
// Graph without optional fields and with edges
let graph: Gfa<()> = Gfa::new();
graph.parse_gfa_file("file.gfa", true); 


// Graph without optional fields and without edges
let graph: Gfa<()> = Gfa::new();
graph.parse_gfa_file("file.gfa", true); 

// Graph without optional fields
let graph: Gfa<()> = Gfa::new();
graph.parse_gfa_file("file.gfa", false); 
```

**Gfa vs NCGfa**  
In general there are two representations of the graph. 
1. Gfa: A raw implementation with node identifiers represented as strings. This representation can be used on any gfa file, whether node ids are numeric or not. 
2. N(numeric)C(compact)Gfa: A compact representation with node identifiers represented as usize starting from 1. It was mainly implemented size it is more memory efficient and nodes lookups can be with the index. 

We recommend using NCGfa in every scenario since there are two main advantages:
1. You can convert a Gfa to a NCGfa and get a Vector data reflects new_id (usize) -> old_id (string)
2. For node lookup you don't need get additinal HashMaps or iterate over the whole vector, you can just take the node id minus 1 look it up directly in the vector. 

### Tips
- For any graph-related output which is based on the features of the graph, don't forget to re-convert node id in order they fit to the input graph structure.
- Convert the graph to numeric and compact node ids before parsing. This saves time for parsing and makes computation faster.



**Reading a graph**: 

```
let graph: Gfa<()> = Gfa::new(); 
// Parse with edges
graph.parse_gfa_file("file.gfa", true); 
``` 
