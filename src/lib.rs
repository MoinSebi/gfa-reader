use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::io::{BufWriter, Write};
use std::path::Path as file_path;
use std::str::Split;
use flate2::read::GzDecoder;


#[derive(Debug, Clone, Default)]
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
        let line = line.split("\t").nth(1).unwrap();
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


/// Trait for OptFields
pub trait OptFields: Sized + Default + Clone {

    /// Return a slice over all optional fields. NB: This may be
    /// replaced by an iterator or something else in the future
    fn fields(&self) -> &[OptElem];

    /// Given an iterator over bytestrings, each expected to hold one
    /// optional field (in the <TAG>:<TYPE>:<VALUE> format), parse
    /// them as optional fields to create a collection. Returns `Self`
    /// rather than `Option<Self>` for now, but this may be changed to
    /// become fallible in the future.
    fn parse(input: Split<&str>) -> Self;

    fn new() -> Self;
    fn iter(&self) -> std::slice::Iter<OptElem> {
        self.iter()
    }

}



/// This implementation is useful for performance if we don't actually
/// need any optional fields. () takes up zero space, and all
/// methods are no-ops.
impl OptFields for () {

    fn fields(&self) -> &[OptElem] {
        &[]
    }

    fn parse(_input: Split<&str> ) -> Self
    {
    }

    fn new() -> Self {
    }
    fn iter(&self) -> std::slice::Iter<OptElem> {
        self.iter()
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
    fn parse(mut input: Split<&str> ) -> Self{
        let mut fields = Vec::new();

        while let Some(value) = input.next() {

            let mut parts = value.split(':');
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

    fn iter(&self) -> std::slice::Iter<OptElem> {
        self.iter()
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

    #[allow(dead_code)]
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
pub struct Containment<T: OptFields>{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
    pub pos : usize, // Position of the overlap
    pub overlap: String,
    pub opt: T,
}

impl <T: OptFields>Containment<T> {

    #[allow(dead_code)]
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
    pub overlap: String,
    pub opt: T,
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



pub trait IsPath {
    fn get_name(&self) -> &String;
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

impl IsPath for Path{
    fn get_name(&self) -> &String{
        &self.name
    }
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
pub struct Gfa<T: OptFields>{
    pub nodes: Vec<Node<T>>,
    pub paths: Vec<Path>,
    pub edges: Option<Vec<Edge<T>>>,
    pub header: Header,
    pub string2index: HashMap<String, usize>,
}



impl <T: OptFields> Gfa <T>{
    /// Graph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::Gfa;
    /// let graph: Gfa<()> = Gfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            paths: Vec::new(),
            edges: None,
            header: Header{tag: "".to_string(), typ: "".to_string(), version_number: "".to_string()},
            string2index: HashMap::new(),

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
            let _f = numeric_nodes.windows(2).all(|pair| pair[1] == &pair[0] + 1);

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
    pub fn parse_gfa_file(&mut self, file_name: &str, edge: bool) {


        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");

            // Parse plain text or gzipped file
            let reader: Box<dyn BufRead> = if file_name.ends_with(".gz") {
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };


            let mut nodes: Vec<Node<T>> = Vec::new();
            let mut edges: Vec<Edge<T>> = Vec::new();
            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let l2 = l.clone();
                let mut a = l2.split("\t");
                let first = a.next().unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match first {
                    "S" => {


                        nodes.push(Node { id: a.next().unwrap().parse().unwrap(), seq:  a.next().unwrap().parse().unwrap(), opt: T::parse(a) });


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
                        if edge {

                            //edges.push(Edge{from: line_split[1].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to: line_split[3].parse().unwrap(), to_dir: if line_split[4] == "+" { !false } else { !true }, overlap: line_split[5].parse().unwrap(), opt: T::parse(line_split)});
                            edges.push(Edge{from: a.next().unwrap().to_string(), from_dir: if a.next().unwrap() == "+" { !false } else { !true }, to: a.next().unwrap().to_string(), to_dir: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a)});

                        }

                    }
                    "C" => {
                        if edge {
                            edges.push(Edge{from: a.next().unwrap().to_string(), from_dir: if a.next().unwrap() == "+" { !false } else { !true }, to: a.next().unwrap().to_string(), to_dir: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a)});

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
            if edge {
                self.edges = Some(edges);
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
        match &self.edges {
            Some(value) =>{
                for edge in value.iter() {
                    write!(f, "{}\n", edge.to_string_link()).expect("Not able to write");
                }
            }
            _ => {}
        }
        for path in self.paths.iter() {
            write!(f, "{}\n", path.to_string()).expect("Not able to write");
        }
    }


    pub fn convert_to_ncgraph(& self, graph: &Gfa<T>, edge: bool) -> NCGfa<T>{
        let mut ncgraph: NCGfa<T> = NCGfa::new();
        let f = ncgraph.make_mapper(graph);
        ncgraph.convert_with_mapper(f, &graph);
        ncgraph
    }
}





/// GFA wrapper
///
/// This is important for PanSN graphs
/// Since the node space is the same, only path need to be merged (which can be done easily)
pub struct GraphWrapper<'a, T: IsPath>{
    pub genomes: Vec<(String, Vec<&'a T>)>,
    pub path2genome: HashMap<&'a String, String>
}


impl <'a, T: IsPath> GraphWrapper<'a, T>{
    pub fn new() -> Self{
        Self{
            genomes: Vec::new(),
            path2genome: HashMap::new(),
        }
    }


    /// GFA -> Wrapper
    /// If delimiter == " " (nothing)
    ///     -> No merging
    pub fn from_gfa(& mut self, paths: &'a Vec<T>, del: &str) {
        let mut name2pathvec: HashMap<String, Vec<&'a T>> = HashMap::new();
        if del == " " {
            for path in paths.iter() {
                name2pathvec.insert(path.get_name().clone(), vec![path]);
            }
        } else {
            for path in paths.iter() {
                let name_split: Vec<&str> = path.get_name().split(del).collect();
                let name_first = name_split[0].clone();
                if name2pathvec.contains_key(&name_first.to_owned().clone()) {
                    name2pathvec.get_mut(&name_first.to_owned().clone()).unwrap().push(path)
                } else {
                    name2pathvec.insert(name_first.to_owned().clone(), vec![path]);
                }
            }
        }
        let mut name2path_value: Vec<(String, Vec<&'a T>)> = Vec::new();
        let mut path_names: Vec<String> = name2pathvec.keys().cloned().collect();
        path_names.sort();
        for path_name in path_names.iter(){
            name2path_value.push((path_name.clone(), name2pathvec.get(path_name).unwrap().clone()));
        }
        let mut name2group = HashMap::new();
        for (name, group) in name2path_value.iter(){
            for path in group.iter(){
                name2group.insert(path.get_name(), name.to_owned());
            }
        }
        self.path2genome = name2group;
        self.genomes = name2path_value;
    }
}


//-------------------------------------------------------------------------------------------------------------------------------------------------





#[derive(Debug, Clone)]
/// The graph contains of nodes, path and edges. NCGfa = NumericCompactGfa
/// This is a compact graph representation of the GFA file.
///
/// Comment: Implementation here are much faster and do include some derivates of parser and data
/// structures that are not parsing the whole file and/or are faster with the downside of more
/// memory.
pub struct NCGfa<T: OptFields>{
    pub header: Header,
    pub nodes: Vec<NCNode<T>>,
    pub paths: Vec<NCPath>,
    pub edges: Option<Vec<NCEdge<T>>>,
    pub mapper: Vec<String>
}



#[derive(Debug, Clone)]
/// Graph nodes:
/// - Identifier
/// - Sequence
/// - Optional elements
pub struct NCNode<T: OptFields>{
    pub id: u32,
    pub seq: String,
    pub opt: T,
}


impl <T: OptFields>NCNode<T> {

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

    #[allow(dead_code)]
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
pub struct NCEdge<T: OptFields>{
    pub from: u32,
    pub from_dir: bool,
    pub to: u32,
    pub to_dir: bool,
    pub overlap: String,
    pub opt: T,
}


impl <T: OptFields>NCEdge<T>  {
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

#[derive(Debug, Clone)]
/// Path features:
/// - names
/// - Directions of the nodes
/// - Node names
pub struct NCPath {
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<u32>,
    pub overlap: Vec<String>,

}

impl NCPath{
    pub fn to_string(&self, mapper: &Option<Vec<&String>>) -> String{
        let a = format!("P\t{}\t", self.name);
        let vec: Vec<String>;
        if Some(mapper) != None{
            vec = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", mapper.as_ref().unwrap()[*n.0 as usize], {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();

        } else {
            vec = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();

        }

        let f2 = vec.join(",");
        format!("{}\t{}\n", a, f2)

    }


    fn to_string2(&self) -> String {
        let a = format!("P\t{}\t", self.name);
        let f1: Vec<String> = self.nodes.iter().zip(&self.dir).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
        let f2 = f1.join(",");
        let f: Vec<String> = self.overlap.iter().map(|a| a.to_string()).collect();
        let g = f.join(",");
        format!("{}\t{}\t{}\n", a, f2, g)
    }
}

impl IsPath for NCPath{
    fn get_name(&self) -> &String{
        &self.name
    }
}

impl  <T: OptFields>NCGfa <T> {

    /// NGraph constructor
    ///
    /// # Example
    ///
    /// ```
    /// use gfa_reader::NCGfa;
    /// let graph: NCGfa<()> = NCGfa::new();
    ///
    /// ```
    pub fn new() -> Self {
        Self {
            header: Header {
                tag: "".to_string(),
                typ: "".to_string(),
                version_number: "".to_string(),
            },
            nodes: Vec::new(),
            paths: Vec::new(),
            edges: None,
            mapper: Vec::new(),
        }
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
    pub fn parse_gfa_file_direct(&mut self, file_name: &str, edge: bool) {


        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");

            // Parse plain text or gzipped file
            let reader: Box<dyn BufRead> = if file_name.ends_with(".gz") {
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };


            let mut nodes: Vec<NCNode<T>> = Vec::new();
            let mut edges: Vec<NCEdge<T>> = Vec::new();

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match line_split[0] {
                    "S" => {

                        let mut a = l.split("\t");
                        a.next();

                        nodes.push(NCNode { id: a.next().unwrap().parse().unwrap(), seq:  a.next().unwrap().parse().unwrap(), opt: T::parse(a) });


                    },
                    "P" => {

                        let name: String = String::from(line_split[1]);
                        let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                        let node_id: Vec<u32> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                        let overlap;
                        if line_split.len() > 3{
                            overlap = line_split[3].split(",").map(|d| d.parse().unwrap()).collect();
                        } else {
                            overlap  = vec!["*".to_string(); node_id.len()];
                        }
                        self.paths.push(NCPath { name: name, dir: dirs, nodes: node_id, overlap: overlap});



                    },
                    "L" => {

                        if edge {
                            let mut a = l.split("\t");
                            a.next();
                            //edges.push(Edge{from: line_split[1].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to: line_split[3].parse().unwrap(), to_dir: if line_split[4] == "+" { !false } else { !true }, overlap: line_split[5].parse().unwrap(), opt: T::parse(line_split)});
                            edges.push(NCEdge{from: a.next().unwrap().parse().unwrap(), from_dir: if a.next().unwrap() == "+" { !false } else { !true }, to: a.next().unwrap().parse().unwrap(), to_dir: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a)});

                        }

                    }
                    "C" => {
                        if edge {
                            let mut a = l.split("\t");
                            a.next();
                            //edges.push(Edge{from: line_split[1].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to: line_split[3].parse().unwrap(), to_dir: if line_split[4] == "+" { !false } else { !true }, overlap: line_split[5].parse().unwrap(), opt: T::parse(line_split)});
                            edges.push(NCEdge{from: a.next().unwrap().parse().unwrap(), from_dir: if a.next().unwrap() == "+" { !false } else { !true }, to: a.next().unwrap().parse().unwrap(), to_dir: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a)});

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
            nodes.sort_by_key(|a| a.id);
            self.nodes.extend(nodes);

        }

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
    pub fn parse_gfa_file_and_convert(&mut self, file_name: &str, edges: bool) {

        let mut graph: Gfa<T> = Gfa::new();
        graph.parse_gfa_file(file_name, edges);
        let ncgraph: NCGfa<T> = graph.convert_to_ncgraph(&graph, edges, );
        self.header = ncgraph.header;
        self.nodes = ncgraph.nodes;
        self.edges = ncgraph.edges;
        self.paths = ncgraph.paths;

    }

    /// Create a mapper
    pub fn make_mapper(&mut self, graph: & Gfa<T>) -> HashMap<String, usize> {
        let mut f = graph.nodes.iter().map(|x| x.id.clone()).collect::<Vec<String>>();
        f.sort_by_key(|digit| digit.parse::<u32>().unwrap());
        let mut wrapper = HashMap::new();
        for (i, node) in f.iter().enumerate() {
            wrapper.insert(node.clone(), i+1);
        }
        wrapper
    }

    /// Convert the "old" graph with the mapper
    pub fn convert_with_mapper(&mut self, mapper: HashMap<String, usize>, graph: &Gfa<T>){
        let mut nodes: Vec<NCNode<T>> = graph.nodes.iter().map(|x| NCNode{id: mapper.get(&x.id).unwrap().clone() as u32, seq: x.seq.clone(), opt: x.opt.clone()}).collect();
        nodes.sort_by_key(|a| a.id);
        self.nodes = nodes;
        self.edges = None;
        match &graph.edges{
            Some(value) => {
                self.edges = Some(value.iter().map(|x| NCEdge{from: mapper.get(&x.from).unwrap().clone() as u32, from_dir: x.from_dir.clone(), to: mapper.get(&x.to).unwrap().clone() as u32, to_dir: x.to_dir.clone(), overlap: "".to_string(), opt: x.opt.clone() }).collect());

            }
            _ =>{}
        }
        self.paths = graph.paths.iter().map(|x| NCPath{name: x.name.clone(), dir: x.dir.clone(), nodes: x.nodes.iter().map(|y| mapper.get(y).unwrap().clone() as u32).collect(), overlap: x.overlap.clone() }).collect();
        let mut test: Vec<(&usize, String)> = mapper.iter().map(|a| (a.1, a.0.clone())).collect();
        test.sort_by_key(|a| a.0);
        self.mapper = test.iter().map(|a| a.1.clone()).collect();

    }

    pub fn get_old_node(&self, node_id: &usize) -> &String{
        &self.mapper[node_id-1]
    }

    /// Write the graph to a file
    pub fn to_file(self, file_name: &str){
        let f = File::create(file_name).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(f, "{}\n",  self.header.to_string1()).expect("Not able to write");
        for node in self.nodes.iter() {
            write!(f, "{}\n", node.to_string()).expect("Not able to write");
        }
        match &self.edges {
            Some(value) =>{
                for edge in value.iter() {
                    write!(f, "{}\n", edge.to_string_link()).expect("Not able to write");
                }
            }
            _ => {}
        }
        for path in self.paths.iter() {
            write!(f, "{}\n", path.to_string2()).expect("Not able to write");
        }
    }
}


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
//

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

/// Check if the vector starts at 1
pub fn vec_check_start(node: &Vec<usize>) -> bool{
    let mm = node.iter().cloned().min().unwrap();
    if mm == 1 {
        return true
    }
    return false
}

/// Create a numeric vector from a vector of strings
pub fn create_sort_numeric(nodes: &Vec<&str>) -> Vec<usize> {
    let mut numeric_nodes = nodes.iter().map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();
    numeric_nodes.sort();
    numeric_nodes
}


/// Check if the vector is compact
pub fn vec_is_compact(numeric_nodes: &Vec<usize>) -> bool{
    numeric_nodes.windows(2).all(|pair| pair[1] == &pair[0] + 1)
}




// Have a collection of intervals and query and find those intervals that overlap with the query. Use crates if possible. Create a function to do this




