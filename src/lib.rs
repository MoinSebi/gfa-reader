use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::io::{BufWriter, Write};
use std::path::Path as file_path;
use flate2::read::GzDecoder;
use std::time::{Duration, Instant};


#[derive(Debug)]
/// GFA header line
/// This line begins with an 'H'
pub struct Header {
    pub tag: String,
    pub typ: String,
    pub version_number: String,
}

impl Header {
    /// Write header to string
    fn to_string1(&self) -> String {
        format!("H\tVN:Z:\t{}", self.version_number)
    }

    /// Parse header from string (H-line)
    fn from_string(line: &str) -> Header {
        let tag = line.split(':').nth(0).unwrap().to_string();
        let typ = line.split(':').nth(1).unwrap().to_string();
        let version_number = line.split(':').nth(2).unwrap().to_string();
        Header {tag, typ, version_number }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Optional fields for GFA 1
pub struct OptElem {
    pub key: String,
    pub typ: String,
    pub val: String,
}

impl OptElem {
    /// Write optional field to string
    fn to_string1(&self) -> String{
        format!("{}\t{}\t{}", self.key, self.typ, self.val)
    }
}



pub trait OptFields: Sized + Default + Clone {

    /// Return a slice over all optional fields. NB: This may be
    /// replaced by an iterator or something else in the future
    fn fields(&self) -> &[OptElem];

    /// Given an iterator over bytestrings, each expected to hold one
    /// optional field (in the <TAG>:<TYPE>:<VALUE> format), parse
    /// them as optional fields to create a collection. Returns `Self`
    /// rather than `Option<Self>` for now, but this may be changed to
    /// become fallible in the future.
    fn parse(input: Vec<&str>) -> Self;

    fn new() -> Self;


}

/// This implementation is useful for performance if we don't actually
/// need any optional fields. () takes up zero space, and all
/// methods are no-ops.
impl OptFields for () {

    fn fields(&self) -> &[OptElem] {
        &[]
    }

    fn parse(_input: Vec<&str>) -> Self
    {
    }

    fn new() -> Self {
    }

}

/// Stores all the optional fields in a vector. `get_field` simply
/// uses std::iter::Iterator::find(), but as there are only a
/// relatively small number of optional fields in practice, it should
/// be efficient enough.
impl OptFields for Vec<OptElem> {
    fn fields(&self) -> &[OptElem] {
        self.as_slice()
    }
    fn parse(input: Vec<&str>) -> Self{
        if input.len() < 4 {
            return Vec::new();
        }
        let mut fields = Vec::new();
        for field in input {
            let mut parts = field.split(':');
            let tag = parts.next().unwrap();
            let typ = parts.next().unwrap();
            let val = parts.next().unwrap();
            fields.push(OptElem {key: tag.to_string(), typ: typ.to_string(), val: val.to_string()});
        }
        fields
    }

    fn new() -> Self {
        Vec::new()
    }
}


pub trait IsEdges<T: OptFields>: Sized + Default + Clone{

    /// Return a slice over all optional fields. NB: This may be
    /// replaced by an iterator or something else in the future
    fn fields(&self) -> Option<&Edge<T>>;

    /// Given an iterator over bytestrings, each expected to hold one
    /// optional field (in the <TAG>:<TYPE>:<VALUE> format), parse
    /// them as optional fields to create a collection. Returns `Self`
    /// rather than `Option<Self>` for now, but this may be changed to
    /// become fallible in the future.
    fn parse(input: Vec<&str>) -> Self;

    fn new() -> Self;

    fn to_string(&self) -> String;


}


/// This implementation is useful for performance if we don't actually
/// need any optional fields. () takes up zero space, and all
/// methods are no-ops.
impl <T: OptFields>IsEdges<T> for () {

    fn fields(&self) -> Option<&Edge<T>> {
        None
    }

    fn parse(_input: Vec<&str>) -> Self
    {
    }

    fn new() -> Self {
    }

    fn to_string(&self) -> String {"".to_string()
    }
}

/// Stores all the optional fields in a vector. `get_field` simply
/// uses std::iter::Iterator::find(), but as there are only a
/// relatively small number of optional fields in practice, it should
/// be efficient enough.
impl <T: OptFields> IsEdges <T> for Edge<T> {

    fn new() -> Self {
        Edge{from: "".to_string(), from_dir: false, to: "".to_string(), to_dir: false, overlap: "".to_string(), opt: T::new(), type_: EdgeType::Link, pos: 0}
    }
    fn fields(&self) -> Option<&Edge<T>>{
        Some(self)
    }

    fn parse(input: Vec<&str>) -> Self{
        Edge{from: "".to_string(), from_dir: false, to: "".to_string(), to_dir: false, overlap: "".to_string(), opt: T::new(), type_: EdgeType::Link, pos: 0}
    }

    fn to_string(&self) -> String {
        self.to_string_link()
    }
}




pub trait OptSeq: Sized + Default + Clone {

    /// Return a slice over all optional fields. NB: This may be
    /// replaced by an iterator or something else in the future
    fn fields(&self) -> &str;

    /// Given an iterator over bytestrings, each expected to hold one
    /// optional field (in the <TAG>:<TYPE>:<VALUE> format), parse
    /// them as optional fields to create a collection. Returns `Self`
    /// rather than `Option<Self>` for now, but this may be changed to
    /// become fallible in the future.
    fn parse(input: Vec<&str>) -> Self;


}


/// Having sequence or not
impl OptSeq for () {

    fn fields(&self) -> &str {
        ""
    }

    fn parse(_input: Vec<&str>) -> Self
    {
    }

}

/// If you dont want to store
impl OptSeq for String {

    fn fields(&self) -> &str {
        self.as_ref()
    }

    fn parse(input: Vec<&str>) -> Self{
        let seq = input[3].to_string();
        seq
    }
}




#[derive(Debug)]
/// Graph nodes:
/// - Identifier
/// - Sequence
/// - Optional elements
pub struct Node<T: OptFields>{
    pub id: String,
    pub seq: String,
    pub opt: T,
}

impl <T: OptFields>Node<T> {

    /// Write node to string
    fn to_string(&self) -> String {
        let a = format!("S\t{}\t{}\n", self.id, self.seq.len());

        if self.opt.fields().len() > 0 {
            let b: Vec<String> = self.opt.fields().iter().map(|a| a.to_string1()).collect();
            let c = b.join("\t");
            format!("{}{}\n", a, c)
        } else {
            a
        }
    }

    /// Write node to fasta
    fn to_fasta(&self) -> String {

        format!(">{}\n{}", self.id, self.seq)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
/// Graph edges
/// - From
/// - From direction
/// - To
/// - To direction
/// - Overlap (Link + containment)
/// - Pos
/// - Ops
///
/// Comment:
/// Edges go forward (true) or backward (false) to/from a node.
pub struct Edge<T: OptFields>{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
    pub pos : usize, // Position of the overlap
    pub overlap: String,
    pub opt: T,
    pub type_: EdgeType,
}

#[derive(Debug, PartialEq, Clone, Default)]
/// Data type for edge type
pub enum EdgeType {
    #[default]
    Link,
    Containment,
    Other
}

impl <T: OptFields>Edge<T> {

    /// Write edge to string
    fn to_string_link(&self) -> String {
        let a = format!("L\t{}\t{}\t{}\t{}\t{}\n", self.from, {if self.from_dir{"+"} else {"-"}}, self.to, {if self.to_dir{"+"} else {"-"}}, self.overlap);
        if self.opt.fields().len() > 0 {
            let b: Vec<String> = self.opt.fields().iter().map(|a| a.to_string1()).collect();
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
/// - Overlap
///
/// Comment: When there is not that many paths, the amount of memory for the overlap is not that much.
pub struct Path{
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<String>,
    pub overlap: Vec<String>,
}

impl Path {

    /// Write path to string (GFA1 format)
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
/// The gfa contains
/// - header
/// - nodes
/// - paths
/// - edges
///
/// Comment: This implementation should be able to parse any kind of GFAv1, but has increased
/// memory consumption, since most node ids are stored at Strings which are a minimum of 24 bytes.
/// This is only maintained, since it is not of further use in any of my projects.
pub struct Gfa<T: OptFields, S: IsEdges<T>>{
    pub nodes: Vec<Node<T>>,
    pub paths: Vec<Path>,
    pub edges: Vec<S>,
    pub header: Header,
}



impl <T: OptFields, S: IsEdges<T>>Gfa <T, S>{
    /// Graph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::Gfa;
    /// let graph: Gfa<(), ()> = Gfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            paths: Vec::new(),
            edges: Vec::new(),
            header: Header{tag: "".to_string(), typ: "".to_string(), version_number: "".to_string()}

        }
    }

    /// Check if the nodes in the graph are
    /// - Nodes are present
    /// - Numeric
    /// - Compact
    /// - Start at 1
    ///
    /// Returns:
    ///     - Option<Vec<usize>>: Nodes ids in usize (from String)
    ///     - Option<usize>: The minimum node id
    pub fn check_nc(&mut self) -> Option<Vec<usize>>{

        // If the graph has no nodes -> returns false
        if self.nodes.len() == 0 {
            return None
        }


        // Check if the graph is numeric

        let is_digit = self.nodes.iter().map(|x| x.id.chars().map(|g| g.is_ascii_digit()).collect::<Vec<bool>>().contains(&false)).collect::<Vec<bool>>().contains(&false);

        // Check if the numeric nodes are compact
        if is_digit {
            let mut numeric_nodes = self.nodes.iter().map(|x| x.id.parse::<usize>().unwrap()).collect::<Vec<usize>>();
            numeric_nodes.sort();
            let f = numeric_nodes.windows(2).all(|pair| pair[1] == pair[0] + 1);

            // Check the min
            let mm = numeric_nodes.iter().cloned().min().unwrap();
            if mm == 1 {
                return Some(numeric_nodes)
            }
        }
            return None


    }






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

            // Parse plain text or gzipped file
            let reader: Box<dyn BufRead> = if file_name.ends_with(".gz") {
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };


            let mut nodes: Vec<Node<T>> = Vec::new();

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match line_split[0] {
                    "S" => {


                        nodes.push(Node { id: line_split[1].to_string(), seq: line_split[2].to_string(), opt: T::parse(line_split) });


                    },
                    "P" => {

                        let name: String = String::from(line_split[1]);
                        let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                        let node_id: Vec<String> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                        let overlap;
                        if line_split.len() > 3{
                            overlap = line_split[3].split(",").map(|d| d.parse().unwrap()).collect();
                        } else {
                            overlap  = vec!["*".to_string(); node_id.len()];
                        }
                        self.paths.push(Path { name: name, dir: dirs, nodes: node_id, overlap: overlap});



                    },
                    "L" => {

                        let mut edge = Edge { from: "".to_string(), to: "".to_string(), from_dir: false, to_dir: false, overlap: "0".to_string(), opt: Vec::new(), type_: EdgeType::Link, pos: 0};
                        let a = S::parse(line_split);
                        if let Some(value) = a.fields(){
                            self.edges.push(a);
                        }
                        //
                        // edge.from = line_split[1].parse().unwrap();
                        // edge.to = line_split[3].parse().unwrap();
                        // edge.to_dir = if line_split[4] == "+" { !false } else { !true };
                        // edge.from_dir = if line_split[2] == "+" { !false } else { !true };
                        // edge.overlap = line_split[5].parse().unwrap();
                        // edge.type_ = EdgeType::Link;
                        // if line_split.len() > 6 {
                        //     for x in line_split.iter().skip(6){
                        //         let mut opt: OptElem = OptElem { key: "".to_string(), typ: "".to_string(), val: "".to_string() };
                        //         let opt_split: Vec<&str> = x.split(":").collect();
                        //         opt.key = opt_split[0].to_string();
                        //         opt.typ = opt_split[1].to_string();
                        //         opt.val = opt_split[2].to_string();
                        //         edge.opt.push(opt);
                        //     }
                        // }


                        // self.edges.push(edge);
                    }
                    "C" => {
                        // let ll: usize = line_split[5].parse().unwrap();
                        // let mut edge = Edge { from: "".to_string(), to: "".to_string(), from_dir: false, to_dir: false, overlap: "0".to_string(), opt: Vec::new(), type_: EdgeType::Link, pos: 0};
                        // edge.from = line_split[1].parse().unwrap();
                        // edge.to = line_split[3].parse().unwrap();
                        // edge.to_dir = if line_split[4] == "+" { !false } else { !true };
                        // edge.from_dir = if line_split[2] == "+" { !false } else { !true };
                        // edge.overlap = line_split[5].parse().unwrap();
                        // edge.type_ = EdgeType::Containment;
                        // edge.pos = ll;
                        // if line_split.len() > 7 {
                        //     for x in line_split.iter().skip(7){
                        //         let mut opt: OptElem = OptElem { key: "".to_string(), typ: "".to_string(), val: "".to_string() };
                        //         let opt_split: Vec<&str> = x.split(":").collect();
                        //         opt.key = opt_split[0].to_string();
                        //         opt.typ = opt_split[1].to_string();
                        //         opt.val = opt_split[2].to_string();
                        //         edge.opt.push(opt);
                        //     }
                        // }
                        let a = S::parse(line_split);
                        if let Some(value) = a.fields(){
                            self.edges.push(a);
                        }
                    }
                    "H" => {
                        let header = Header::from_string(&l);
                        self.header = header;
                    }
                    _ => {
                    }
                }

            }

            self.nodes.extend(nodes);

        }

    }

    /// Write the graph to a file
    pub fn to_file(self, file_name: &str){
        let f = File::create(file_name).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(f, "{}\n",  self.header.to_string1()).expect("Not able to write");
        for node in self.nodes.iter() {
            write!(f, "{}\n", node.to_string()).expect("Not able to write");
        }
        for edge in self.edges.iter() {
            write!(f, "{}\n", edge.to_string()).expect("Not able to write");
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
    pub fn from_gfa(& mut self, paths: &'a Vec<Path>, del: &str) {
        let mut name2pathvec: HashMap<String, Vec<&'a Path>> = HashMap::new();
        if del == " " {
            for path in paths.iter() {
                name2pathvec.insert(path.name.clone(), vec![path]);
            }
        } else {
            for path in paths.iter() {
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

//
//
//
//
// #[derive(Debug, Clone)]
// /// The graph contains of nodes, path and edges. NCGfa = NumericCompactGfa
// /// This is a compact graph representation of the GFA file.
// ///
// /// The goal:
// /// - Direct implementation of map
// ///
// /// Comment: Implementation here are much faster and do include some derivates of parser and data
// /// structures that are not parsing the whole file and/or are faster with the downside of more
// /// memory.
// pub struct NCGfa{
//     pub nodes: Vec<NNode>,
//     pub paths: Vec<NPath>,
//     pub edges: Vec<NEdge>,
//     pub mapper: Option<HashMap<String, usize>>,
// }
//
//
//
// #[derive(Debug, Clone)]
// /// Graph nodes:
// /// - identifier
// /// - sequence
// ///
// /// Comment:
// /// Add opt after
// pub struct NNode {
//     pub id: u32,
//     pub seq: String,
// }
//
//
// impl NNode {
//     pub fn new()-> Self{
//         Self{
//             id: 0,
//             seq: String::new(),
//         }
//     }
//
//     pub fn to_string(&self, mapper: &Option<Vec<&String>>) -> String {
//         let a;
//         if Some(mapper) != None{
//             a = format!("S\t{}\t{}\n", self.id, self.seq.len());
//         } else {
//             a = format!("S\t{}\t{}\n", mapper.as_ref().unwrap()[self.id as usize], self.seq.len());
//         }
//         a
//     }
// }
//
// #[derive(Debug, Clone)]
// /// Graph edges
// /// - from
// /// - from direction
// /// - to
// /// - to direction
// ///
// /// Comment:
// /// Edges go forward (true) or backward (false) to/from a node.
// pub struct NEdge {
//     pub from: u32,
//     pub from_dir: bool,
//     pub to: u32,
//     pub to_dir: bool,
// }
//
// impl NEdge {
//     pub fn to_string(&self, mapper: &Option<Vec<String>>) -> String{
//         let a ;
//         if Some(mapper) != None{
//             a = format!("L\t{}\t{}\t{}\t{}\n", mapper.as_ref().unwrap()[self.from as usize].clone(), {if self.from_dir{"+"} else {"-"}}, mapper.as_ref().unwrap()[self.to as usize].clone(), {if self.to_dir{"+"} else {"-"}});
//         } else {
//             a = format!("L\t{}\t{}\t{}\t{}\n", self.from, {if self.from_dir{"+"} else {"-"}}, self.to, {if self.to_dir{"+"} else {"-"}});
//         }
//         a
//     }
//
// }
//
// #[derive(Debug, Clone)]
// /// Path features:
// /// - names
// /// - Directions of the nodes
// /// - Node names
// pub struct NPath {
//     pub name: String,
//     pub dir: Vec<bool>,
//     pub nodes: Vec<u32>,
//
// }
//
// impl NPath{
//     pub fn to_string(&self, mapper: &Option<Vec<&String>>) -> String{
//         let a = format!("P\t{}\t", self.name);
//         let vec: Vec<String>;
//         if Some(mapper) != None{
//             vec = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", mapper.as_ref().unwrap()[*n.0 as usize], {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
//
//         } else {
//             vec = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
//
//         }
//
//         let f2 = vec.join(",");
//         format!("{}\t{}\n", a, f2)
//
//     }
// }
//
// impl NCGfa {
//
//     /// NGraph constructor
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use gfa_reader::NCGfa;
//     /// let graph = NCGfa::new();
//     ///
//     /// ```
//     pub fn new() -> Self {
//
//         Self {
//             nodes: Vec::new(),
//             paths: Vec::new(),
//             edges: Vec::new(),
//             mapper: None,
//         }
//     }
//
//     /// Mapper<&String, usize> -> Vec<&String>
//     ///
//     /// Reverse the mapper to vector
//     pub fn reverse_mapper(&self) -> Vec<String>{
//
//         // When mapper ->
//         let mut usize2id = vec!["".to_string(); self.nodes.len()+1];
//
//         if self.mapper.is_some(){
//             for x in self.mapper.as_ref().unwrap().iter(){
//                 usize2id[*x.1] = x.0.to_string();
//             }
//         } else {
//             usize2id = self.nodes.iter().map(|x| x.id.to_string()).collect();
//             usize2id.insert(0, "".to_string());
//             }
//
//
//         usize2id
//     }
//
//     /// Get the nnode from the real node_id
//     pub fn get_real_node(&self, node_id: &String) -> bool{
//         if Some(self.mapper.as_ref().unwrap()) != None {
//             let st = self.mapper.as_ref().unwrap().get(node_id).unwrap();
//             let node = &self.nodes[*st].clone();
//         } else {
//             let node = &self.nodes[node_id.parse::<usize>().unwrap()];
//         }
//         false
//     }
//
//     /// NGraph constructor when feature sizes are known
//     /// Useful when converting from normal graph to this kind of graph.
//     /// # Example
//     ///
//     /// ```
//     /// use gfa_reader::NCGfa;
//     /// let graph = NCGfa::with_capacity(10,10,10);
//     ///
//     /// ```
//     pub fn with_capacity(nodes_number: usize, paths_number: usize, edge_number: usize) -> Self {
//         Self {
//             nodes: Vec::with_capacity(nodes_number),
//             paths: Vec::with_capacity(paths_number),
//             edges: Vec::with_capacity(edge_number),
//             mapper: None,
//         }
//     }
//
//
//     /// Convert normal gfa to NCGFA
//     pub fn from_gfa_struct<T: OptFields, S: IsEdges>(&mut self, graph: &mut Gfa<T, S>) {
//         let a = graph.check_nc();
//         if a != None{
//             let mut nodes: Vec<NNode> = Vec::with_capacity(a.unwrap().len());
//             graph.nodes.iter().for_each(|x| nodes[x.id.parse::<usize>().unwrap()] = NNode{id: x.id.parse::<u32>().unwrap(), seq: x.seq.clone()});
//             self.edges = graph.edges.iter().map(|x| NEdge{from: x.from.parse().unwrap(), from_dir: x.from_dir, to: x.to.parse().unwrap(), to_dir: x.to_dir}).collect();
//             self.paths = graph.paths.iter().map(|x| NPath{name: x.name.clone(), dir: x.dir.clone(), nodes: x.nodes.iter().map(|y| y.parse().unwrap()).collect()}).collect();
//         } else {
//             let a = self.make_mapper(graph);
//             self.convert_with_mapper(a, graph);
//         }
//     }
//
//     /// Create mapper from old node id to new id
//     pub fn make_mapper<T: OptFields, S: IsEdges>(&mut self, graph: &mut Gfa<T, S>) -> HashMap<String, usize> {
//         let mut f = graph.nodes.iter().map(|x| x.id.clone()).collect::<Vec<String>>();
//         f.sort_by_key(|digit| digit.parse::<u32>().unwrap());
//         let mut wrapper = HashMap::new();
//         for (i, node) in f.iter().enumerate() {
//             wrapper.insert(node.clone(), i+1);
//         }
//         wrapper
//     }
//
//     /// Convert the "old" graph with the mapper
//     pub fn convert_with_mapper<T: OptFields, S: IsEdges>(&mut self, mapper: HashMap<String, usize>, graph: &Gfa<T, S>){
//         self.nodes = graph.nodes.iter().map(|x| NNode{id: *mapper.get(&x.id).unwrap() as u32, seq: x.seq.clone()}).collect();
//         self.edges = graph.edges.iter().map(|x| NEdge{from: *mapper.get(&x.from).unwrap() as u32, from_dir: x.from_dir, to: *mapper.get(&x.to).unwrap() as u32, to_dir: x.to_dir}).collect();
//         self.paths = graph.paths.iter().map(|x| NPath{name: x.name.clone(), dir: x.dir.clone(), nodes: x.nodes.iter().map(|y| *mapper.get(y).unwrap() as u32).collect()}).collect();
//         self.mapper = Some(mapper)
//
//     }
//
//
//
//
//
//
//
//
//
//     /// Parse GFA file
//     pub fn parse_gfa_file(&mut self, filename: &str) -> bool{
//         if file_path::new(filename).exists() {
//             let mut file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
//             let mut contents = String::new();
//             file.read_to_string(&mut contents).unwrap();
//
//             // Temp nodes
//             let mut nodes = Vec::new();
//
//             for line in contents.lines() {
//                 let line_split: Vec<&str> = line.split("\t").collect();
//                 match line_split[0] {
//                     "S" => {
//                         let id = line_split[1].parse().unwrap();
//                         nodes.push(NNode { id: id, seq: line_split[2].to_string()});}
//                     "P" =>{
//                         let name: String = String::from(line_split[1]);
//                         let c = line_split[2].split(",");
//                         let mut dirs: Vec<bool> = c.clone().map(|d| &d[d.len() - 1..] == "+" ).collect();
//                         let mut nodd: Vec<u32> = c.map(|d| d[..d.len() - 1].parse().unwrap()).collect();
//
//                         dirs.shrink_to_fit();
//                         nodd.shrink_to_fit();
//                         self.paths.push(NPath { name, dir: dirs, nodes: nodd });
//
//                     } "L" => {
//                         self.edges.push(NEdge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir:  line_split[2] == "+" , to_dir: line_split[4] == "+" })
//                     }
//                     _ => ()
//                 }
//             }
//             // Sort the keys
//             nodes.sort_by_key(|a| a.id);
//             let nodes_id_max = nodes[nodes.len()-1].id;
//             let mut new_nodes = vec![NNode::new(); nodes_id_max as usize + 1];
//
//             nodes.into_iter().for_each(|x| new_nodes[x.id as usize] = x.clone());
//             self.nodes = new_nodes;
//             self.nodes.shrink_to_fit();
//             self.edges.shrink_to_fit();
//             self.paths.shrink_to_fit();
//
//         }
//         false
//     }
// }
//
// /// GFA wrapper
// ///
// /// This is important for PanSN graphs
// /// Since the node space is the same, only path need to be merged (which can be done easily)
// pub struct NCGraphWrapper<'a>{
//     pub genomes: Vec<(String, Vec<&'a NPath>)>,
//     pub path2genome: HashMap<&'a String, String>
// }
//
//
// impl <'a> NCGraphWrapper<'a>{
//
//     /// Constructor (empty)
//     pub fn new() -> Self{
//         Self{
//             genomes: Vec::new(),
//             path2genome: HashMap::new(),
//         }
//     }
//
//
//
//
//     /// GFA -> Wrapper
//     /// If delimiter == " " (nothing)
//     ///     -> No merging
//     pub fn from_ngfa(& mut self, graph: &'a NCGfa, del: &str) {
//         let mut name2pathvec: HashMap<String, Vec<&'a NPath>> = HashMap::new();
//         if del == " " {
//             for path in graph.paths.iter() {
//                 name2pathvec.insert(path.name.clone(), vec![path]);
//             }
//         } else {
//             for path in graph.paths.iter() {
//                 let name_split: Vec<&str> = path.name.split(del).collect();
//                 let name_first = name_split[0].clone();
//                 if name2pathvec.contains_key(&name_first.to_owned().clone()) {
//                     name2pathvec.get_mut(&name_first.to_owned().clone()).unwrap().push(path)
//                 } else {
//                     name2pathvec.insert(name_first.to_owned().clone(), vec![path]);
//                 }
//             }
//         }
//         let mut name2path_value: Vec<(String, Vec<&'a NPath>)> = Vec::new();
//         let mut path_names: Vec<String> = name2pathvec.keys().cloned().collect();
//         path_names.sort();
//         for path_name in path_names.iter(){
//             name2path_value.push((path_name.clone(), name2pathvec.get(path_name).unwrap().clone()));
//         }
//         let mut name2group = HashMap::new();
//         for (name, group) in name2path_value.iter(){
//             for path in group.iter(){
//                 name2group.insert(&path.name, name.to_owned());
//             }
//         }
//         self.path2genome = name2group;
//         self.genomes = name2path_value;
//     }
// }
//
//
//
// /// Check if a file has compact and numeric nodes
// pub fn read_nodes(filename: &str) -> bool{
//     if file_path::new(filename).exists() {
//         let mut file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
//         let mut contents = String::new();
//         file.read_to_string(&mut contents).unwrap();
//
//         // path name -> path_number
//         let mut nodes = Vec::new();
//
//         for line in contents.lines() {
//             let line_split: Vec<&str> = line.split("\t").collect();
//             match line_split[0] {
//                 "S" => {
//                     nodes.push(line_split[1]);
//                 }
//                 _ => ()
//             }
//         }
//         let is_digit = vec_is_digit(&nodes);
//         if is_digit {
//             let numeric_nodes = create_sort_numeric(&nodes);
//             let compact = vec_is_compact(&numeric_nodes);
//             return compact
//         }
//         return true
//     }
//     return false
// }

/// Does this vector contain only digits
pub fn vec_is_digit(nodes: &Vec<&str>) -> bool{

    nodes.iter().map(|x| x.chars().map(|g| g.is_ascii_digit()).collect::<Vec<bool>>().contains(&false)).collect::<Vec<bool>>().contains(&false)
}

/// Create a numeric vector from a vector of strings
pub fn create_sort_numeric(nodes: &Vec<&str>) -> Vec<usize> {
    let mut numeric_nodes = nodes.iter().map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();
    numeric_nodes.sort();
    numeric_nodes
}

pub fn vec_is_compact(numeric_nodes: &Vec<usize>) -> bool{
    numeric_nodes.windows(2).all(|pair| pair[1] == pair[0] + 1)
}




// Have a collection of intervals and query and find those intervals that overlap with the query. Use crates if possible. Create a function to do this




