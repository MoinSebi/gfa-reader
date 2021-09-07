use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::hash::Hash;

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

impl Gfa{
    pub fn new() -> Self{
        let nodes: HashMap<String, Node> = Hash::new();
        let paths: Vec<Path> = Vec::new();
        let edges: Vec<Edge> = Vec::new();
        Self {
            nodes: nodes,
            paths: paths,
            edges: edges,
        }
    }

    pub fn read_file(& mut self, file_name: &str){
        let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
        let reader = BufReader::new(file);

        // Iterate over lines
        for line in reader.lines() {
            let l = line.unwrap();
            let line_split: Vec<&str> = l.split("\t").collect();
            if l.starts_with("S") {
                if self.nodes.contains_key(&String::from(line_split[1])){
                    eprintln!("Warining: Duplicated node if found");
                }
                self.nodes.insert(String::from(line_split[1]), Node { id: String::from(line_split[1]), seq: String::from(line_split[2]), len: line_split[2].len() });

            }
            else if l.starts_with("P"){
                let name: String = String::from(line_split[1]);
                let dirs: Vec<bool> = line_split[2].split(",").map(|d| if &d[d.len()-1..] == "+" { !false } else { !true }).collect();
                let nodd: Vec<String> = line_split[2].split(",").map(|d| d[..d.len()-1].parse().unwrap()).collect();
                paths.push(Path {name: name, dir: dirs, nodes: nodd});
            }
            else if l.starts_with("L") {
                edges.push(Edge{from: line_split[1].parse().unwrap() , to: line_split[3].parse().unwrap() , from_dir: if line_split[2] == "+" { !false } else { !true }, to_dir: if line_split[4] == "+" { !false } else { !true }})

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

