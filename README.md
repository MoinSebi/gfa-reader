# gfa-reader - Reading GFA files

Able to work with version  1.0, 1.1, 1.2 in plain text format. Newer version will be supported in the future. 
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


#### Supported sample types:
As stated above, we support the following sample types:
- String
- u32
- u64
- SeqIndex

Comment: Use "String" only if it is not possible to use the other types and the IDs must be represented as strings. Alternatively use SeqIndex which is smaller but can also be used as a string.

#### Overlaps
Overlaps are optional: () or SeqIndex. 
#### Optional fields
Overlaps are optional: () or SeqIndex.



```rust
let mut graph: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/size5.gfa");
```
## PanSN
Pan-SN spec is a specification for storing variation graphs in a GFA format. It is strongly supported by gfa-reader with a pansn struct. It allows you to utilize genome, haplotype or path level collections, dependent on the use case.  
If the data is not in pansn format, each path will represent its own genome and haplotypes. Take care that you have the same delimiter twice. 


## Walks
Walks can be interpreted as "alternative representation" of paths in the graph. We can convert walks to path using PanSN-spec. The start and and end of the walk are concatenated at the end of the path name. We add a non-existing Overlap "*" as the for the path, since this information is not given in the walk specification. 


