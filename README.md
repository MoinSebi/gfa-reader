# gfa-reader - Reading GFA files

Able to work with version  1.0, 1.1, 1.2 in plain text format. Newer version will be supported in the future. This is read only - graphs (at least the sequence) can not change using this implementation. Nevertheless, this graph representation is extremely memory efficient. 
- v1: https://gfa-spec.github.io/GFA-spec/GFA1.html
- v2: https://gfa-spec.github.io/GFA-spec/GFA2.html  


### Installation
```
gfa-readder = { git = "https://github.com/MoinSebi/gfa-reader", branch = "main" }
```
**OR** 
```
gfa-reader = "0.1.5"
```
### General information:
Gfa-reader has one main structure: Gfa. It contains three generics which can be adjusted. 
1. Sample ID: The type of the sample id. Can be a String, u32, u64, SeqIndex
2. Overlap information: () or SeqIndex
3. Optional fields: () or SeqIndex
4. 
```
let graph: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/size5.gfa");
```

#### Supported sample types:
As stated above, we support the following sample types:
- String
- u32
- u64
- usize
- SeqIndex

**Comment**: Use "String" only if it is not possible to use the other types and the IDs must be represented as strings. Alternatively use SeqIndex which is smaller but can also return "&str" information. 

#### Overlaps
Overlaps are optional, since many graphs construction pipelines do not return graphs with overlaps.  
Possible values: () or SeqIndex. 
#### Optional fields
Optionals fields in GFA can contain powerfull information. There is no additional parsing of these fields, except holding the raw String.  
Possible values: () or SeqIndex.

## PanSN
Pan-SN spec is a specification for storing paths in GFA format. It is strongly supported by ```gfa-reader``` with a ```Pansn``` struct. It allows you to utilize genome, haplotype or path level, dependent on the use case (see below).   
The hierachy is the following: A genome is a collection of different haplotypes, which are a collection of multiple paths. The Pansn struct contains of a vector of genomes. If the data is not in PanSn-spec, each path will represent in its own genome (1-1-1).

```text
let wrapper: Pansn<u32, (), ()> = Pansn::from_graph(&graph.paths, "#");

//Extracting collection of path on different levels: 
let haplo_paths: Vec<(String, &Path)> = wrapper.get_haplo_path(); 
let genome_paths: Vec<(String, &Path)> = wrapper.get_path_genome(); 
let all_paths: Vec<(String, &Path)> = wrapper.get_paths_directly();
```

## Walks
Walks can be interpreted as "alternative representation" of paths. We can convert walks to path using PanSN-spec by creating a specific path name using the information provided by the walk. The start and end of ```walk``` are concatenated at the end of the path name. We add a non-existing overlap "*" in each newly created path, since this information is not given in the walk specification. 

```text
let graph: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/size5.gfa");
graph.walk_to_path("#");
```

## Additional Gfa/Pansn functions
- ```gfa.get_node_by_id(node_id) -> &Segement``` returns a specific node by its ID using binary search.

## Additional GFA related functions 
- ```Gfa::check_numeric_gfafile()``` checks if the GFA file contains numeric node IDs.


