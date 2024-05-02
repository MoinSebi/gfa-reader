use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::io::{BufWriter, Write};
use std::path::Path as file_path;
use std::str::Split;
use std::thread::sleep;
use std::time::Duration;
use memmap2::Mmap;
use rand::distributions::Open01;

#[derive(Debug, Clone, Default, Ord, PartialEq, Eq, PartialOrd)]
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
        format!("H\tVN:Z:\t{}\n", self.version_number)
    }

    /// Parse header from string (H-line)
    fn from_string(line: &str) -> Header {
        let line = line.split_whitespace().nth(1).unwrap();
        let tag = line.split(':').nth(0).unwrap().to_string();
        let typ = line.split(':').nth(1).unwrap().to_string();
        let version_number = line.split(':').nth(2).unwrap().to_string();
        Header {
            tag,
            typ,
            version_number,
        }
    }
}

pub trait SampleType {
    fn parse1(input: &str, s: &mut String) -> Self;
}

impl SampleType for String{
    fn parse1(input: &str, s: &mut String) -> Self{
        s.to_string()
    }
}

impl SampleType for u64{
    fn parse1(input: &str, s: &mut String) -> Self{
        input.parse().unwrap()
    }
}
impl SampleType for u32{
    fn parse1(input: &str, s: &mut String) -> Self{
        input.parse().unwrap()
    }
}

impl SampleType for seq_index{
    fn parse1(input: &str, s: &mut String) -> Self{
        s.push_str(input);
        Self([s.len(), s.len() - input.len()])
    }
}


pub trait Opt {
    fn parse1(input: Option<&str>, s: &mut String) -> Self;
}

impl Opt for (){
    fn parse1(input: Option<&str>, s: &mut String) -> Self{
        ()
    }
}

impl Opt for seq_index{
    fn parse1(input: Option<&str>, s: &mut String) -> Self{
        if input.is_none(){
            return seq_index([0, 0]);
        } else {
            let input = input.unwrap();
            s.push_str(input);
            seq_index([s.len(), s.len() - input.len()])
        }
    }
}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct seq_index([usize; 2]);

impl seq_index{
    fn parse1(input: &str, s: &mut String) -> Self{

        s.push_str(input);
        Self([s.len(), s.len() - input.len()])
    }

    fn get_string<'a >(&self, s: &'a String) -> &'a str{
        &s[self.0[0]..self.0[1]]
    }

}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Segment<T: SampleType + Ord, S: Opt + Ord> {
    pub id: T,
    pub sequence: seq_index,
    pub length: u32,
    pub opt: S,
}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Link<T: SampleType, S: Opt, U: Opt>{
    pub from: T,
    pub from_dir: bool,
    pub to: T,
    pub to_dir: bool,
    pub overlap: U,
    pub opt: S,
}
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Path<T: SampleType, S: Opt, U: Opt>{
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<T>,
    pub overlap: U,
    pub opt: S,
}
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Walk<T: SampleType, S: Opt>{
    pub sample_id: String,
    pub hap_index: u32,
    pub seq_id: String,
    pub seq_start: i32,
    pub seq_end: i32,
    pub walk_dir: Vec<bool>,
    pub walk_id: Vec<T>,
    pub opt: S,
}
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Containment<T: SampleType, S: Opt>{
    pub container: T,
    pub container_dir: bool,
    pub contained: T,
    pub contained_dir: bool,
    pub pos: u32,
    pub overlap: seq_index,
    pub opt: S,
}
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Jump<T: SampleType, S: Opt>{
    pub from: T,
    pub from_dir: bool,
    pub to: T,
    pub to_dir: bool,
    pub distance: i64,
    pub opt: S,
}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Gfa<T: SampleType + Ord, S: Opt + Ord, U: Opt>{
    pub header: Header,
    pub segments: Vec<Segment<T, S>>,
    pub links: Vec<Link<T, S, U>>,
    pub paths: Vec<Path<T, S, U>>,
    pub jump: Vec<Jump<T, S>>,
    pub containment: Vec<Containment<T, S>>,
    pub walk: Vec<Walk<T, S>>,


    pub sequence: String

}

impl <T: SampleType + Ord + Clone, S: Opt + Ord + Clone, U: Opt> Gfa<T, S, U>{
    pub fn new() -> Self {
        Self {
            header: Header::default(),
            segments: Vec::new(),
            sequence: String::new(),
            links: Vec::new(),
            paths: Vec::new(),
            jump: Vec::new(),
            containment: Vec::new(),
            walk: Vec::new(),
        }
    }

    pub fn parse_gfa_file(file_name: &str) -> Gfa<T, S, U>{
        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            let version_number = get_version(file_name);
            let mut z: Gfa<T, S, U> = Gfa::new();

            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let mut a = l.split('\t');
                match a.next().unwrap() {
                    "S" => {
                        let name = a.next().unwrap();
                        if version_number < 2.0 {
                            let sequence = a.next().unwrap();
                            let size = sequence.len() as u32;
                            let opt = a.next();
                            z.segments.push(Segment {
                                id: T::parse1(name, &mut z.sequence),
                                sequence: seq_index::parse1(sequence, &mut z.sequence),
                                length: size,
                                opt: S::parse1(opt, &mut z.sequence),
                            });
                        } else {
                            let sequence = a.next().unwrap();
                            let size = a.next().unwrap().parse().unwrap();
                            let opt = a.next();

                            z.segments.push(Segment {
                                id: T::parse1(name, &mut z.sequence),
                                sequence: seq_index::parse1(sequence, &mut z.sequence),
                                length: size,
                                opt: S::parse1(opt, &mut z.sequence),

                            });
                        }
                    }
                    "H" => {
                        let header = Header::from_string(&l);
                        z.header = header;
                    }
                    "L" => {
                        let from = a.next().unwrap();
                        let from_dir = a.next().unwrap() == "+";
                        let to = a.next().unwrap();
                        let to_dir = a.next().unwrap() == "+";
                        let overlap = a.next();
                        let opt = a.next();
                        z.links.push(Link {
                            from: T::parse1(from, &mut z.sequence),
                            from_dir,
                            to: T::parse1(to, &mut z.sequence),
                            to_dir,
                            overlap: U::parse1(overlap, &mut z.sequence),
                            opt: S::parse1(opt, &mut z.sequence),
                        });
                    }
                    "P" => {
                        let name = a.next().unwrap().to_string();
                        let (dirs, node_id) = path_parser(a.next().unwrap(), &mut z.sequence);
                        let overlap = a.next();
                        z.paths.push(Path {
                            name,
                            dir: dirs,
                            nodes: node_id,
                            overlap: U::parse1(overlap, &mut z.sequence),
                            opt: S::parse1(a.next(), &mut z.sequence),
                        });
                    }
                    "W" => {
                        let sample_id = a.next().unwrap().to_string();
                        let hap_index = a.next().unwrap().parse().unwrap();
                        let seq_id = a.next().unwrap().to_string();
                        let seq_start = a.next().unwrap().parse().unwrap();
                        let seq_end = a.next().unwrap().parse().unwrap();
                        let (w1, w2) = walk_parser(a.next().unwrap(), &mut z.sequence);
                        let opt = a.next();
                        z.walk.push(Walk {
                            sample_id,
                            hap_index,
                            seq_id,
                            seq_start,
                            seq_end,
                            walk_dir: w1,
                            walk_id: w2,
                            opt: S::parse1(opt, &mut z.sequence),
                        });
                    }
                    "C" => {
                        let container = a.next().unwrap();
                        let container_dir = a.next().unwrap() == "+";
                        let contained = a.next().unwrap();
                        let contained_dir = a.next().unwrap() == "+";
                        let pos = a.next().unwrap().parse().unwrap();
                        let overlap = a.next().unwrap();
                        let opt = a.next();
                        z.containment.push(Containment {
                            container: T::parse1(container, &mut z.sequence),
                            container_dir,
                            contained: T::parse1(contained, &mut z.sequence),
                            contained_dir,
                            pos,
                            overlap: seq_index::parse1(overlap, &mut z.sequence),
                            opt: S::parse1(opt, &mut z.sequence),
                        });
                    }
                    "J" => {
                        let from = a.next().unwrap();
                        let from_dir = a.next().unwrap() == "+";
                        let to = a.next().unwrap();
                        let to_dir = a.next().unwrap() == "+";
                        let distance = parse_dumb(a.next().unwrap());
                        let opt = a.next();
                        z.jump.push(Jump {
                            from: T::parse1(from, &mut z.sequence),
                            from_dir,
                            to: T::parse1(to, &mut z.sequence),
                            to_dir,
                            distance,
                            opt: S::parse1(opt, &mut z.sequence),
                        });
                    }
                    _ => {}
                }
            }
            z.segments.sort();
            z
        }
        else {
            panic!("ERROR: FILE NOT FOUND\n");
        }

    }
    pub fn walk_to_path(&mut self){

        for walk in self.walk.iter(){
            self.paths.push(Path{
                name: walk.sample_id.clone() +"#" +  &walk.hap_index.to_string() +"#"+ &walk.seq_id.clone() +":"+ &walk.seq_start.to_string() +"-"+ &walk.seq_end.to_string(),
                dir: walk.walk_dir.clone(),
                nodes: walk.walk_id.iter().cloned().collect(),
                overlap: U::parse1(None, &mut self.sequence),
                opt: walk.opt.clone(),
            });
        }
        self.walk = Vec::new();


    }

    pub fn is_compact(&self) -> bool{
        self.segments[0].id == T::parse1("1", &mut String::new()) && self.segments[self.segments.len()-1].id == T::parse1(&self.segments.len().to_string(), &mut String::new())
    }
}



pub fn get_version(file_name: &str) -> f32 {
    let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
    let reader = BufReader::new(file);
    let mut version_number = 0.0;
    for line in reader.lines() {
        let l = line.unwrap();
        if l.starts_with("H") {
            let a = l.split_whitespace().nth(1).unwrap();
            version_number = a.split(':').nth(2).unwrap().parse().unwrap();
            break;
        }
    }
    version_number
}


fn path_parser<T: SampleType>(path: &str, s: &mut String) ->(Vec<bool>, Vec<T>){
    let a = path.split(",");
    let (mut dirs, mut node_id) = (Vec::with_capacity(a.clone().count()), Vec::with_capacity(a.clone().count()));
    for d in a{
        dirs.push(&d[d.len() - 1..] == "+");
        node_id.push(SampleType::parse1(&d[..d.len() - 1], s));
    }
    (dirs, node_id)
}

fn walk_parser<T: SampleType>(walk: &str, s1: &mut String) ->(Vec<bool>, Vec<T>) {
    let a = walk[1..].split(['<', '>']).count();
    let (mut dirs, mut node_id) = (Vec::with_capacity(a), Vec::with_capacity(a));
    dirs.push(walk.chars().next().unwrap() == '>');
    let mut s = String::new();
    for x in walk[1..].chars(){
        if x == '<' || x == '>'{
            dirs.push(x == '>');
            node_id.push(T::parse1(&s, s1));
            s = String::new();
        } else {
            s.push(x);
        }
    }
    (dirs, node_id)
}

fn parse_dumb(s: &str) -> i64{
    if s == "*"{
        return -1;
    } else {
        return s.parse().unwrap();
    }
}







