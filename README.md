# gfaR - gfa reader

Simple library for gfa files

**Installation**:
``` 
cargo install gfaR

[dependencies]
gfaR = "0.1.2"
```


**Reading a graph**: 

```
let graph = Gfa::new(); 
graph.read_file("file.gfa"); 
``` 
