use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path as file_path;

#[cfg(test)]

pub fn counter(start: i32, end: i32){
    for index in start..end{
        println!("C: {}", index);
    }
}

// Basic and most important
// TODO
// Missing values


#[derive(Debug)]
pub struct Node{
    pub id: String,
    pub len: usize,
    pub seq: String,
}

#[derive(Debug)]
pub struct Edge{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
}

#[derive(Debug)]
pub struct Path{
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<String>,
}

#[derive(Debug)]
pub struct Gfa{
    pub nodes: HashMap<String, Node>,
    pub paths: Vec<Path>,
    pub edges: Vec<Edge>,
}


#[derive(Debug, Clone)]
/// The NumericGFA is GFA were node_ids are interger (u32)
pub struct NGfa{
    pub nodes: HashMap<u32, NNode>,
    pub paths: Vec<NPath>,
    pub edges: Vec<NEdge>,
    pub path2id: HashMap<String, usize>,
}


#[derive(Debug, Clone)]
pub struct NNode {
    pub id: u32,
    pub len: usize,
    pub seq: String,
}

#[derive(Debug, Clone)]
pub struct NEdge {
    pub from: u32,
    pub from_dir: bool,
    pub to: u32,
    pub to_dir: bool,
}

#[derive(Debug, Clone)]
pub struct NPath {
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<u32>,

}

impl NGfa {
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
    pub fn from_file_direct(&mut self, filename: &str) {
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);
            let mut count = 0;
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                if l.starts_with("S") {
                    if self.nodes.contains_key(&line_split[1].parse::<u32>().unwrap()) {
                        eprintln!("Warning: Duplicated node id found");
                    }
                    self.nodes.insert(line_split[1].parse().unwrap(), NNode { id: line_split[1].parse().unwrap(), seq: String::from(line_split[2]), len: line_split[2].len() });
                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    self.path2id.insert(name.clone(), count);
                    count += 1;
                    let mut dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                    let mut nodd: Vec<u32> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();

                    self.paths.push(NPath { name: name, dir: dirs, nodes: nodd });
                } else if l.starts_with("L") {
                    self.edges.push(NEdge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to_dir: if line_split[4] == "+" { !false } else { !true } })
                }
            }
        }

        self.nodes.shrink_to_fit();
        self.edges.shrink_to_fit();
        self.paths.shrink_to_fit();
    }

    pub fn from_file_direct2(&mut self, filename: &str) {
        let mut nodes = vec![];
        let mut path2id = vec![];
        if file_path::new(filename).exists() {
            let file = File::open(filename).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);
            let mut count = 0;

            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match line_split[0]{
                    "S" => {nodes.push((line_split[1].parse::<u32>().unwrap(), NNode{id: line_split[1].parse::<u32>().unwrap(), seq:  "".to_string(), len: line_split[2].len()}));
                }, "P" =>  {
                        let name = String::from(line_split[1]);
                    path2id.push((name.clone(), count));
                    count += 1;
                    let mut dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { true } else { false }).collect();
                    let mut nodd: Vec<u32> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    dirs.shrink_to_fit();
                    nodd.shrink_to_fit();
                    self.paths.push(NPath { name: String::from(name), dir: dirs, nodes: nodd });
                }
                    &_ => {}
                }
            }
        }
        self.path2id.extend(path2id.into_iter());
        self.nodes.extend(nodes.into_iter());
        self.paths.shrink_to_fit();
        self.nodes.shrink_to_fit();
    }
}

impl Gfa {
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

    pub fn read_file(&mut self, file_name: &str) {
        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                if l.starts_with("S") {
                    if self.nodes.contains_key(&String::from(line_split[1])) {
                        eprintln!("Warning: Duplicated node id found");
                    }
                    self.nodes.insert(String::from(line_split[1]), Node { id: String::from(line_split[1]), seq: String::from(line_split[2]), len: line_split[2].len() });
                } else if l.starts_with("P") {
                    let name: String = String::from(line_split[1]);
                    let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len() - 1..] == "+" { !false } else { !true }).collect();
                    let nodd: Vec<String> = line_split[2].split(",").map(|d| d[..d.len() - 1].parse().unwrap()).collect();
                    self.paths.push(Path { name: name, dir: dirs, nodes: nodd });
                } else if l.starts_with("L") {
                    self.edges.push(Edge { from: line_split[1].parse().unwrap(), to: line_split[3].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to_dir: if line_split[4] == "+" { !false } else { !true } })
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    // cargo test -- --nocapture
    #[test]
    fn basic() {
        // We test remove and and general function

    }
}

