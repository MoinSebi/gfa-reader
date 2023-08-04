# gfa-reader - gfa reader

Library for reading gfa (v1) files. 

```
git clone https://github.com/MoinSebi/gfa-reader
```

**Reading a graph**: 

```
let graph: Gfa<()> = Gfa::new(); 
// Parse with edges
graph.parse_gfa_file("file.gfa", true); 
``` 
