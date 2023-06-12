use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::io::{BufWriter, Write};
use std::path::Path as file_path;
use std::process::id;
use std::ptr::read;
use flate2::read::GzDecoder;


#[derive(Debug)]
/// GFA 1/2 header line
/// This line begins with an 'H'
pub struct Header {
    pub version_number: String,
}

impl Header {
    fn to_string2(&self) -> String {
        format!("H\t{}", self.version_number)
    }

    fn from_string(line: &str) -> Header {
        let mut line = line.split_whitespace();
        line.next();
        let version_number = line.next().unwrap().to_string();
        Header { version_number }
    }
}

#[derive(Debug, PartialEq)]
/// Optional fields for GFA 1/2
pub struct opt_elem{
    pub key: String,
    pub typ: String,
    pub val: String,
}

impl opt_elem{
    fn to_string1(&self) -> String{
        format!("{}\t{}\t{}", self.key, self.typ, self.val)
    }
}




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
    pub seq: String,
    pub opt: Vec<opt_elem>,
}

impl Node {

    // Write node to string
    fn to_string(&self) -> String {
        let a = format!("S\t{}\t{}\n", self.id, self.seq.len());
        if self.opt.len() > 0 {
            let b: Vec<String> = self.opt.iter().map(|a| a.to_string1()).collect();
            let c = b.join("\t");
            format!("{}{}\n", a, c)
        } else {
            a
        }
    }

    // Write node to fasta
    fn to_fasta(&self) -> String {
        format!(">{}\n{}", self.id, self.seq)
    }
}

#[derive(Debug, PartialEq)]
/// Graph edges
/// - from
/// - from direction
/// - to
/// - to direction
/// - Overlap (Link + containment)
/// - Pos
/// - Ops
///
/// Comment:
/// Edges go forward (true) or backward (false) to/from a node.
pub struct Edge{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
    pub pos : usize, // Position of the overlap
    pub overlap: String,
    pub opt: Vec<opt_elem>,
    pub type_: EdgeType,
}

#[derive(Debug, PartialEq)]
/// Data type for edge type
pub enum EdgeType {
    Link,
    Containment,
    Other
}

impl Edge {

    // Write edge to string
    fn to_string_link(&self) -> String {
        let a = format!("L\t{}\t{}\t{}\t{}\t{}\n", self.from, {if self.from_dir{"+"} else {"-"}}, self.to, {if self.to_dir{"+"} else {"-"}}, self.overlap);
        if self.opt.len() > 0 {
            let b: Vec<String> = self.opt.iter().map(|a| a.to_string1()).collect();
            let c = b.join("\t");
            format!("{}{}\n", a, c)
        } else {
            a
        }
    }
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
    pub overlap: Vec<String>,
}

impl Path {

    // Write path to string (GFA1 format)
    fn to_string(&self) -> String {
        let a = format!("P\t{}\t", self.name);
        let f1: Vec<String> = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
        let f2 = f1.join(",");
        let f: Vec<String> = self.overlap.iter().map(|a| a.to_string()).collect();
        let g = f.join(",");
        format!("{}\t{}\t{}\n", a, f2, g)
    }
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
    pub header: Header,
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
        let header = Header{ version_number: "VN:Z:1.0".to_string() };
        Self {
            nodes: nodes,
            paths: paths,
            edges: edges,
            header: header

        }
    }
    // Open a gzip file and import the crate




    /// Read the graph from a file
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// use gfa_reader::Gfa;
    /// let mut graph = Gfa::new();
    /// graph.parse_gfa_file("/path/to/graph");
    /// ´´´
    pub fn parse_gfa_file(&mut self, file_name: &str) {
        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");

            let reader: Box<dyn BufRead> = if file_name.ends_with(".gz") {
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };

            let mut nodes = Vec::new();

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();

                // If line is segment
                if line_split[0] == "S" {
                    let mut node: Node = Node { id: "".to_string(), seq: "".to_string(), opt: Vec::new() };
                    node.seq = line_split[2].to_string();
                    node.id = line_split[1].to_string();
                    node.opt = Vec::new();

                    if line_split.len() > 3 {
                        for x in line_split.iter().skip(3){
                            let mut opt: opt_elem = opt_elem{ key: "".to_string(), typ: "".to_string(), val: "".to_string() };
                            let opt_split: Vec<&str> = x.split(":").collect();
                            opt.key = opt_split[0].to_string();
                            opt.typ = opt_split[1].to_string();
                            opt.val = opt_split[2].to_string();
                            node.opt.push(opt);
                        }
                    }
                    nodes.push((node.id.clone(), node));

                } else if l.starts_with("L") {
                    let mut edge = Edge { from: "".to_string(), to: "".to_string(), from_dir: false, to_dir: false, overlap: "0".to_string(), opt: Vec::new(), type_: EdgeType::Link, pos: 0};
                    edge.from = line_split[1].parse().unwrap();
                    edge.to = line_split[3].parse().unwrap();
                    edge.to_dir = if line_split[4] == "+" { !false } else { !true };
                    edge.from_dir = if line_split[2] == "+" { !false } else { !true };
                    edge.overlap = line_split[5].parse().unwrap();
                    edge.type_ = EdgeType::Link;
                    if line_split.len() > 6 {
                        for x in line_split.iter().skip(6){
                            let mut opt: opt_elem = opt_elem{ key: "".to_string(), typ: "".to_string(), val: "".to_string() };
                            let opt_split: Vec<&str> = x.split(":").collect();
                            opt.key = opt_split[0].to_string();
                            opt.typ = opt_split[1].to_string();
                            opt.val = opt_split[2].to_string();
                            edge.opt.push(opt);
                        }
                    }
                    self.edges.push(edge);
                } else if l.starts_with("C ") {
                    let ll: usize = line_split[5].parse().unwrap();
                    let mut edge = Edge { from: "".to_string(), to: "".to_string(), from_dir: false, to_dir: false, overlap: "0".to_string(), opt: Vec::new(), type_: EdgeType::Link, pos: 0};
                    edge.from = line_split[1].parse().unwrap();
                    edge.to = line_split[3].parse().unwrap();
                    edge.to_dir = if line_split[4] == "+" { !false } else { !true };
                    edge.from_dir = if line_split[2] == "+" { !false } else { !true };
                    edge.overlap = line_split[5].parse().unwrap();
                    edge.type_ = EdgeType::Containment;
                    edge.pos = ll;
                    if line_split.len() > 7 {
                        for x in line_split.iter().skip(7){
                            let mut opt: opt_elem = opt_elem{ key: "".to_string(), typ: "".to_string(), val: "".to_string() };
                            let opt_split: Vec<&str> = x.split(":").collect();
                            opt.key = opt_split[0].to_string();
                            opt.typ = opt_split[1].to_string();
                            opt.val = opt_split[2].to_string();
                            edge.opt.push(opt);
                        }
                    }
                    self.edges.push(edge);


                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                    let node_id: Vec<String> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    let mut overlap = Vec::new();
                    if line_split.len() > 3{
                        overlap = line_split[3].split(",").map(|d| d.parse().unwrap()).collect();
                    } else {
                        overlap  = vec!["*".to_string(); node_id.len()];
                    }
                    self.paths.push(Path { name: name, dir: dirs, nodes: node_id, overlap: overlap})


                } else if l.starts_with("H") {
                    self.header = Header { version_number: String::from(line_split[1]) };
                }

            }
            self.nodes = HashMap::with_capacity(nodes.len());
            self.nodes.extend(nodes.into_iter());

        }
    }

    /// Write the graph to a file
    pub fn to_file(self, file_name: &str){
        let f = File::create(file_name).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(f, "{}\n",  self.header.to_string2());
        for node in self.nodes.iter() {
            write!(f, "{}\n", node.1.to_string()).expect("Not able to write");
        }
        for edge in self.edges.iter() {
            write!(f, "{}\n", edge.to_string_link()).expect("Not able to write");
        }
        for path in self.paths.iter() {
            write!(f, "{}\n", path.to_string()).expect("Not able to write");
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
        new_graph.nodes.insert(x.0.parse().unwrap(), NNode { seq: x.1.seq, len: 0, id: x.1.id.parse().unwrap() });
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
    //#[test]
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

    //#[test]
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

    //#[test]
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

    //#[test]
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

