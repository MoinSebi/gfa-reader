use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

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


pub struct Gfa{
    pub nodes: HashMap<String, Node>,
    pub paths: Vec<Path>,
    pub edges: Vec<Edge>,
}

pub fn readGFA(a: &str) -> Gfa {

    // This is the reader structure
    // "/home/svorbrugg_local/Rust/data/AAA_AAB.cat.gfa";
    let file_name= a;
    let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
    let reader = BufReader::new(file);

    // Things to return, mut data

    let mut ns: HashMap<String, Node> = HashMap::new();
    let mut ps: Vec<Path> = Vec::new();
    let mut es: Vec<Edge> = Vec::new();




    // Iterate over lines
    for line in reader.lines() {
        let l = line.unwrap();
        let lsplit: Vec<&str> = l.split("\t").collect();
        if l.starts_with("S") {
            ns.insert(String::from(lsplit[1]), Node { id: String::from(lsplit[1]), seq: String::from(lsplit[2]), len: lsplit[2].len() });

        }
        else if l.starts_with("P"){
            let name: String = String::from(lsplit[1]);
            let dirs: Vec<bool> = lsplit[2].split(",").map(|d| if &d[d.len()-1..] == "+" { !false } else { !true }).collect();
            let nodd: Vec<String> = lsplit[2].split(",").map(|d| d[..d.len()-1].parse().unwrap()).collect();
            ps.push(Path {name: name, dir: dirs, nodes: nodd});
        }
        else if l.starts_with("L") {
            es.push(Edge{from: lsplit[1].parse().unwrap() , to: lsplit[3].parse().unwrap() , from_dir: if lsplit[2] == "+" { !false } else { !true }, to_dir: if lsplit[4] == "+" { !false } else { !true }})

        }

    }
    let gs = Gfa{nodes: ns, edges: es, paths: ps};
    gs

}

