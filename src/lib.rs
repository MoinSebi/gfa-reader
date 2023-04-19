use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path as file_path;
use std::process::id;
use std::ptr::read;


#[derive(Debug)]
/// Graph nodes:
/// - identifier
/// - length of the sequence
/// - sequence
///
/// Comment:
/// Sequence is stored as String which is (in most cases) very memory heavy. Future changed might
/// involve just storing [u8]
pub struct Node{
    pub id: String,
    pub len: usize,
    pub seq: String,
}

#[derive(Debug)]
/// Graph edges
/// - from
/// - from direction
/// - to
/// - to direction
///
/// Comment:
/// Edges go forward (true) or backward (false) to/from a node.
pub struct Edge{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
}

#[derive(Debug)]
/// Path features:
/// - names
/// - Directions of the nodes
/// - Node names
pub struct Path{
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<String>,
}



#[derive(Debug)]
/// The graph contains of nodes, path and edges.
/// This is a simple implementation where identifiers of nodes can be of any kind. I highly recommend using integers, but in the format description it is not required.
/// Most of the above structures are very simple and do not contain any flags/overlap information.
/// More might come later
///
/// Comment: This implementation should be able to parse any kind of GFAv1, but has increased
/// memory consumption, since many parts are stored at Strings which are a minimum of 24 bytes.
/// This is only maintained, since it is not of further use in any of my projects.
pub struct Gfa{
    pub nodes: HashMap<String, Node>,
    pub paths: Vec<Path>,
    pub edges: Vec<Edge>,
}


impl Gfa {
    /// Graph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::Gfa;
    /// let graph = Gfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        let nodes: HashMap<String, Node> = HashMap::new();
        let paths: Vec<Path> = Vec::new();
        let edges: Vec<Edge> = Vec::new();
        Self {
            nodes: nodes,
            paths: paths,
            edges: edges,
        }
    }


    /// Read the graph from a file
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// use gfa_reader::Gfa;
    /// let mut graph = Gfa::new();
    /// graph.read_file("/path/to/graph");
    /// ´´´
    pub fn read_file(&mut self, file_name: &str) {
        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                if l.starts_with("S") {
                    self.nodes.insert(String::from(line_split[1]), Node { id: String::from(line_split[1]), seq: String::from(line_split[2]), len: line_split[2].len() });
                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                    let node_id: Vec<String> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    self.paths.push(Path { name: name, dir: dirs, nodes: node_id });
                } else if l.starts_with("L") {
                    self.edges.push(Edge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to_dir: if line_split[4] == "+" { !false } else { !true } })
                }
            }
        }
    }
}

/// GFA wrapper
///
/// This is important for PanSN graphs
/// Since the node space is the same, only path need to be merged (which can be done easily)
pub struct GraphWrapper<'a>{
    pub genomes: Vec<(String, Vec<&'a Path>)>,
    pub path2genome: HashMap<&'a String, String>
}


impl <'a> GraphWrapper<'a>{
    pub fn new() -> Self{
        Self{
            genomes: Vec::new(),
            path2genome: HashMap::new(),
        }
    }



    /// GFA -> Wrapper
    /// If delimiter == " " (nothing)
    ///     -> No merging
    pub fn from_ngfa(& mut self, graph: &'a Gfa, del: &str) {
        let mut name2pathvec: HashMap<String, Vec<&'a Path>> = HashMap::new();
        if del == " " {
            for path in graph.paths.iter() {
                name2pathvec.insert(path.name.clone(), vec![path]);
            }
        } else {
            for path in graph.paths.iter() {
                let name_split: Vec<&str> = path.name.split(del).collect();
                let name_first = name_split[0].clone();
                if name2pathvec.contains_key(&name_first.to_owned().clone()) {
                    name2pathvec.get_mut(&name_first.to_owned().clone()).unwrap().push(path)
                } else {
                    name2pathvec.insert(name_first.to_owned().clone(), vec![path]);
                }
            }
        }
        let mut name2path_value: Vec<(String, Vec<&'a Path>)> = Vec::new();
        let mut path_names: Vec<String> = name2pathvec.keys().cloned().collect();
        path_names.sort();
        for path_name in path_names.iter(){
            name2path_value.push((path_name.clone(), name2pathvec.get(path_name).unwrap().clone()));
        }
        let mut name2group = HashMap::new();
        for (name, group) in name2path_value.iter(){
            for path in group.iter(){
                name2group.insert(&path.name, name.to_owned());
            }
        }
        self.path2genome = name2group;
        self.genomes = name2path_value;
    }
}


//-------------------------------------------------------------------------------------------------------------------------------------------------





#[derive(Debug, Clone)]
/// The graph contains of nodes, path and edges. NGfa = **N**umbericGfa
/// This is a simple implementation where identifiers of nodes required to be unsiged interges (0
/// or bigger). Most of the structures are very simple and do not contain any flags/overlap
/// information.
/// More might come later
///
/// Comment: Implementation here are much faster and do include some derivates of parser and data
/// structures that are not parsing the whole file and/or are faster with the downside of more
/// memory.
pub struct NGfa{
    pub nodes: HashMap<u32, NNode>,
    pub paths: Vec<NPath>,
    pub edges: Vec<NEdge>,
    pub path2id: HashMap<String, usize>,
}


#[derive(Debug, Clone)]
/// Graph nodes:
/// - identifier
/// - length of the sequence
/// - sequence
///
/// Comment:
/// Sequence is stored as String which is (in most cases) very memory heavy. Future changed might
/// involve just storing [u8].
pub struct NNode {
    pub id: u32,
    pub len: usize,
    pub seq: String,
}

#[derive(Debug, Clone)]
/// Graph edges
/// - from
/// - from direction
/// - to
/// - to direction
///
/// Comment:
/// Edges go forward (true) or backward (false) to/from a node.
pub struct NEdge {
    pub from: u32,
    pub from_dir: bool,
    pub to: u32,
    pub to_dir: bool,
}

#[derive(Debug, Clone)]
/// Path features:
/// - names
/// - Directions of the nodes
/// - Node names
pub struct NPath {
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<u32>,

}

impl NGfa {

    /// NGraph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::NGfa;
    /// let graph = NGfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        let nodes: HashMap<u32, NNode> = HashMap::new();
        let paths: Vec<NPath> = Vec::new();
        let edges: Vec<NEdge> = Vec::new();

        Self {
            nodes: nodes,
            paths: paths,
            edges: edges,
            path2id: HashMap::new(),
        }
    }

    /// NGraph constructor when feature sizes are known
    /// Useful when converting from normal graph to this kind of graph.
    /// # Example
    ///
    /// ```
    /// use gfa_reader::NGfa;
    /// let graph = NGfa::with_capacity(10,10,10);
    ///
    /// ```
    pub fn with_capacity(nodes_number: usize, paths_number: usize, edge_number: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(nodes_number),
            paths: Vec::with_capacity(paths_number),
            edges: Vec::with_capacity(edge_number),
            path2id: HashMap::with_capacity(paths_number)
        }
    }


    /// Read NGfa from a file
    pub fn read_file(&mut self, filename: &str) {
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                if l.starts_with("S") {
                    let mut id = line_split[1].parse().unwrap();
                    nodes.push((id, NNode { id: id, seq: String::from(line_split[2]), len: line_split[2].len() }));
                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    self.path2id.insert(name.clone(), count);
                    count += 1;
                    let c = line_split[2].split(",");
                    let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                    let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();
                    self.paths.push(NPath { name: name, dir: dirs, nodes: nodd });

                } else if l.starts_with("L") {
                    self.edges.push(NEdge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir:  line_split[2] == "+" , to_dir: line_split[4] == "+" })
                }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }

    /// NGfa parser_m1
    /// Modified version 1
    /// - Does not read the sequence in the nodes
    /// - Does not read edges at all
    /// Read NGfa from a file
    pub fn read_file_m1(&mut self, filename: &str) {
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                if l.starts_with("S") {
                    let mut id = line_split[1].parse().unwrap();
                    nodes.push((id, NNode { id: id, seq: "".to_string(), len: line_split[2].len() }));
                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    self.path2id.insert(name.clone(), count);
                    count += 1;
                    let c = line_split[2].split(",");
                    let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                    let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();
                    self.paths.push(NPath { name, dir: dirs, nodes: nodd });

                }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }


    pub fn read_file_m2(&mut self, filename: &str) {
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match line_split[0] {
                    "S" => {let mut id = line_split[1].parse().unwrap();
                    nodes.push((id, NNode { id: id, seq: "".to_string(), len: line_split[2].len() }));}
                "P" =>{
                    let name: String = String::from(line_split[1]);
                    self.path2id.insert(name.clone(), count);
                    count += 1;
                    let c = line_split[2].split(",");
                    let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                    let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();
                    self.paths.push(NPath { name, dir: dirs, nodes: nodd });

                } "L" => {
                    self.edges.push(NEdge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir:  line_split[2] == "+" , to_dir: line_split[4] == "+" })
                }
                    _ => ()
            }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }

    pub fn read_file_m3(&mut self, filename: &str) {
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match line_split[0] {
                    "S" => {

                        let mut id = line_split[1].parse().unwrap();
                        nodes.push((id, NNode { id: id, seq: "".to_string(), len: line_split[2].len() }));}
                    "P" =>{
                        let name: String = String::from(line_split[1]);
                        self.path2id.insert(name.clone(), count);
                        count += 1;
                        let c = line_split[2].split(",");
                        let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                        let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                        dirs.shrink_to_fit();
                        nodd.shrink_to_fit();
                        self.paths.push(NPath { name, dir: dirs, nodes: nodd });

                    }
                    _ => ()
                }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }


    pub fn read_file_string(&mut self, filename: &str) {
        //let mut nodes = vec![];
        let mut nodes: Vec<u32> = Vec::new();
        //let mut paths = Vec::new();
        if file_path::new(filename).exists() {
            let mut file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in contents.lines() {
                let line_split: Vec<&str> = line.split("\t").collect();
                match line_split[0] {
                    "S" => {let mut id = line_split[1].parse().unwrap();
                        nodes.push((id, NNode { id: id, seq: "".to_string(), len: line_split[2].len() }));}
                    "P" =>{
                        let name: String = String::from(line_split[1]);
                        self.path2id.insert(name.clone(), count);
                        count += 1;
                        let c = line_split[2].split(",");
                        let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                        let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                        dirs.shrink_to_fit();
                        nodd.shrink_to_fit();
                        self.paths.push(NPath { name, dir: dirs, nodes: nodd });

                    } "L" => {
                        self.edges.push(NEdge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir:  line_split[2] == "+" , to_dir: line_split[4] == "+" })
                    }
                    _ => ()
                }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }
}




use rayon::prelude::*;
pub fn read_file_in_parallel(file_path: &str) {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut nodes: Vec<u32> = Vec::new();

    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let chunk_size = lines.len() / 6;

    lines.par_chunks(chunk_size).for_each(|chunk| {
        let mut nodes: Vec<u32> = Vec::new();
        for line in chunk {
            let line_split: Vec<&str> = line.split("\t").collect();
            match line_split[0] {
                "S" => nodes.push(line_split[1].parse().unwrap()),

                "P " => {
                    let name = String::from(line_split[1]);
                    //let mut faster: Vec<_> = line_split[2].split(",").map(|d| (if &d[d.len() - 1..] == "+" { true } else { false }, d[..d.len() - 1].parse::<u32>().unwrap())).collect();
                    let mut c = line_split[2].split(",");
                    let mut dirs: Vec<_> = c.clone().map(|d| (if &d[d.len() - 1..] == "+" { true } else { false })).collect();
                    let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();
                }
                _ => ()
            }
        }
    });
}



/// Converts a "normal" Gfa to NGfa
///
/// # Example
/// ´´´
/// use gfa_reader::{from_Gfa, Gfa, NGfa};
///
/// let graph: Gfa = Gfa::new();
/// let ngraph: NGfa = from_Gfa(graph);
///
/// ```
pub fn from_Gfa(graph: Gfa) -> NGfa{
    let mut new_graph = NGfa::with_capacity(graph.nodes.len(), graph.paths.len(), graph.edges.len());
    for x in graph.nodes.into_iter() {
        new_graph.nodes.insert(x.0.parse().unwrap(), NNode { seq: x.1.seq, len: x.1.len, id: x.1.id.parse().unwrap() });
    }
    for path in graph.paths.into_iter(){
        let ids: Vec<u32> = path.nodes.into_iter().map(|n| n.parse().unwrap()).collect();
        new_graph.paths.push(NPath{name: path.name, dir: path.dir, nodes: ids})
    }
    for edge in graph.edges.into_iter(){
        new_graph.edges.push(NEdge{from_dir: edge.from_dir, to_dir: edge.to_dir, from: edge.from.parse().unwrap(), to: edge.to.parse().unwrap()})
    }

    new_graph
}



pub struct NNode2 {
    pub id: u32,
    pub len: usize,
}

#[derive(Debug, Clone)]
/// Graph edges
/// - from
/// - from direction
/// - to
/// - to direction
///
/// Comment:
/// Edges go forward (true) or backward (false) to/from a node.
pub struct NEdge2 {
    pub from: u32,
    pub from_dir: bool,
    pub to: u32,
    pub to_dir: bool,
}

#[derive(Debug, Clone)]
/// Path features:
/// - names
/// - Directions of the nodes
/// - Node names
pub struct NPath2 {
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<u32>,

}

pub struct NGfa2{
    pub nodes: HashMap<u32, NNode2>,
    pub paths: Vec<NPath2>,
    pub edges: Vec<NEdge2>,
    pub path2id: HashMap<String, usize>,
}

impl NGfa2 {

    /// NGraph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::NGfa;
    /// let graph = NGfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        let nodes: HashMap<u32, NNode2> = HashMap::new();
        let paths: Vec<NPath2> = Vec::new();
        let edges: Vec<NEdge2> = Vec::new();

        Self {
            nodes: nodes,
            paths: paths,
            edges: edges,
            path2id: HashMap::new(),
        }
    }
    pub fn read_file_string(&mut self, filename: &str) {
        //let mut nodes = vec![];
        let mut nodes: Vec<u32> = Vec::new();
        //let mut paths = Vec::new();
        if file_path::new(filename).exists() {
            let mut file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            // path name -> path_number
            let mut count = 0;
            let mut nodes = Vec::new();

            for line in contents.lines() {
                let line_split: Vec<&str> = line.split("\t").collect();
                match line_split[0] {
                    "S" => {let mut id = line_split[1].parse().unwrap();
                        nodes.push((id, NNode2 { id: id, len: line_split[2].len() }));}
                    "P" =>{
                        let name: String = String::from(line_split[1]);
                        self.path2id.insert(name.clone(), count);
                        count += 1;
                        let c = line_split[2].split(",");
                        let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
                        let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();

                        dirs.shrink_to_fit();
                        nodd.shrink_to_fit();
                        self.paths.push(NPath2 { name, dir: dirs, nodes: nodd });

                    } "L" => {
                        self.edges.push(NEdge2 { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir:  line_split[2] == "+" , to_dir: line_split[4] == "+" })
                    }
                    _ => ()
                }
            }
            self.nodes.extend(nodes.into_iter());
            self.nodes.shrink_to_fit();
            self.edges.shrink_to_fit();
            self.paths.shrink_to_fit();
        }
    }
}



#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::NGfa;

    // cargo test -- --nocapture --test-threads=1
    // --test-threads=1
    #[test]
    fn basic() {
        println!("h");
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        println!("{:?}", since_the_epoch);
        let filename = "/home/svorbrugg/code/bvd/data/example_data/chr1.sort.small.gfa";
        let mut graph = NGfa::new();
        graph.read_file_string(filename);
        println!("Done3");

    }

    #[test]
    fn basic2() {
        println!("h");
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        println!("{:?}", since_the_epoch);
        let filename = "/home/svorbrugg/code/bvd/data/example_data/chr1.sort.small.gfa";
        let mut graph = NGfa::new();
        graph.read_file(filename);
        println!("Done3");

    }

    #[test]
    fn basic3() {
        println!("h");
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        println!("{:?}", since_the_epoch);
        let filename = "/home/svorbrugg/code/bvd/data/example_data/chr1.sort.small.gfa";
        let mut graph = NGfa::new();
        graph.read_file_m1(filename);
        println!("Done3");

    }

    #[test]
    fn basic4() {
        println!("h");
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        println!("{:?}", since_the_epoch);
        let filename = "/home/svorbrugg/code/bvd/data/example_data/chr1.sort.small.gfa";
        let mut graph = NGfa::new();
        graph.read_file_m2(filename);
        println!("Done3");

    }
}

