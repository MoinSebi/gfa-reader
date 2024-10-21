use std::fs::File;
use std::io::{prelude::*, BufReader};

use std::path::Path as file_path;
use std::thread::sleep;

#[derive(Debug, Clone, Default, Ord, PartialEq, Eq, PartialOrd)]
/// GFA header line
/// This line begins with an 'H'
pub struct Header {
    pub tag: String,
    pub typ: String,
    pub version_number: String,
}

impl Header {
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

/// Possible generics which can be used as identifier
pub trait SampleType {
    /// Parse a string to a generic type
    ///
    /// Might use a String to add the relevant data
    fn parse1(input: &str, s: &mut String) -> Self;

    fn get_usize(&self) -> usize;

    fn is_digit() -> bool;
}

impl SampleType for String {
    fn parse1(_input: &str, s: &mut String) -> Self {
        s.to_string()
    }

    fn get_usize(&self) -> usize {
        0
    }

    fn is_digit() -> bool {
        false
    }


}

impl SampleType for usize {
    fn parse1(input: &str, _s: &mut String) -> Self {
        input.parse().unwrap()
    }
    fn get_usize(&self) -> usize {
        *self as usize
    }

    fn is_digit() -> bool {
        true
    }
}

impl SampleType for u64 {
    fn parse1(input: &str, _s: &mut String) -> Self {
        input.parse().unwrap()
    }
    fn get_usize(&self) -> usize {
        *self as usize
    }


    fn is_digit() -> bool {
        true
    }
}
impl SampleType for u32 {
    fn parse1(input: &str, _s: &mut String) -> Self {
        input.parse().unwrap()
    }

    fn get_usize(&self) -> usize {
        *self as usize
    }

    fn is_digit() -> bool {
        true
    }
}

impl SampleType for SeqIndex {
    fn parse1(input: &str, s: &mut String) -> Self {
        s.push_str(input);
        Self([s.len() - input.len(), s.len()])
    }

    fn get_usize(&self) -> usize {
        0
    }

    fn is_digit() -> bool {
        false
    }
}

/// Optional fields
///
/// In addition, used for Overlap fields in the graph
pub trait Opt {
    fn parse1(input: Option<&str>, s: &mut String) -> Self;
}

impl Opt for () {
    fn parse1(_input: Option<&str>, _s: &mut String) -> Self {}
}

impl Opt for SeqIndex {
    fn parse1(input: Option<&str>, s: &mut String) -> Self {
        if input.is_none() {
            SeqIndex([0, 0])
        } else {
            let input = input.unwrap();
            s.push_str(input);
            Self([s.len() - input.len(), s.len()])
        }
    }
}

///  Start position and end position of a sequence
///
/// Similar to a slice
/// Can be sued to get sequence and len
///
/// Memory size: 16 byte
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct SeqIndex([usize; 2]);

impl SeqIndex {
    fn parse1(input: &str, s: &mut String) -> Self {
        s.push_str(input);
        Self([s.len() - input.len(), s.len()])
    }

    pub fn get_string<'a>(&self, s: &'a str) -> &'a str {
        &s[self.0[0]..self.0[1]]
    }

    pub fn get_len(&self) -> usize {
        self.0[1] - self.0[0]
    }
}

/// GFA segment
///
/// Memory size: 16 + 4 + 0 + 0 = 20
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Segment<T: SampleType + Ord, S: Opt + Ord> {
    pub id: T,
    pub sequence: SeqIndex,
    pub length: u32,
    pub opt: S,
}

/// GFA link
///
/// Memory size (u32): 4 + 1 + 4 + 1 + 0 + 0 = 12 (padding)
///
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Link<T: SampleType, S: Opt, U: Opt> {
    pub from: T,
    pub to: T,
    pub from_dir: bool,
    pub to_dir: bool,
    pub overlap: U,
    pub opt: S,
}

/// GFA Path
///
/// Memory size (u32): String + 4*X + 1*X + 0 + 0 ~ 5*x
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Path<T: SampleType, S: Opt, U: Opt> {
    pub name: String,
    pub dir: Vec<bool>,
    pub nodes: Vec<T>,
    pub overlap: U,
    pub opt: S,
}

/// GFA Walk
///
/// Memory size (u32): 5*x
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Walk<T: SampleType, S: Opt> {
    pub sample_id: String,
    pub hap_index: u32,
    pub seq_id: String,
    pub seq_start: i32,
    pub seq_end: i32,
    pub walk_dir: Vec<bool>,
    pub walk_id: Vec<T>,
    pub opt: S,
}

impl<T: SampleType, S: Opt> Drop for Walk<T, S> {
    fn drop(&mut self) {
        println!("Dropping LargeData2 with data_string: {}", self.sample_id);
        // You can add more debug prints or cleanup logic here
    }
}

/// GFA Containment
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Containment<T: SampleType, S: Opt> {
    pub container: T,
    pub container_dir: bool,
    pub contained: T,
    pub contained_dir: bool,
    pub pos: u32,
    pub overlap: SeqIndex,
    pub opt: S,
}

/// GFA Jump
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Jump<T: SampleType, S: Opt> {
    pub from: T,
    pub from_dir: bool,
    pub to: T,
    pub to_dir: bool,
    pub distance: i64,
    pub opt: S,
}

/// Gfa struct
#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Gfa<T: SampleType + Ord, S: Opt + Ord, U: Opt> {
    pub header: Header,
    pub segments: Vec<Segment<T, S>>,
    pub links: Vec<Link<T, S, U>>,
    pub paths: Vec<Path<T, S, U>>,
    pub jump: Vec<Jump<T, S>>,
    pub containment: Vec<Containment<T, S>>,
    pub walk: Vec<Walk<T, S>>,

    pub is_digit: bool,
    index_of_index: Vec<usize>,
    index_low: usize,
    pub sequence: String,
}

impl<T: SampleType + Ord + Clone, S: Opt + Ord + Clone, U: Opt> Default for Gfa<T, S, U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: SampleType + Ord + Clone, S: Opt + Ord + Clone, U: Opt> Gfa<T, S, U> {
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
            is_digit: false,

            index_of_index: Vec::new(),
            index_low: 0,
        }
    }

    /// Parse a GFA file
    pub fn parse_gfa_file(file_name: &str) -> Gfa<T, S, U> {
        if file_path::new(file_name).exists() {
            println!("Reading file: {}", file_name);
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
            let reader = BufReader::new(file);

            let version_number = get_version(file_name);
            let mut z: Gfa<T, S, U> = Gfa::new();

            // Iterate over lines
            for line in reader.lines() {
                let line_string = line.unwrap();
                let mut split_line = line_string.split_whitespace();
                match split_line.next().unwrap() {
                    "S" => {
                        let name = split_line.next().unwrap();
                        if version_number <= 2.0 {
                            let sequence = split_line.next().unwrap();
                            let size = sequence.len() as u32;
                            let opt = split_line.next();
                            z.segments.push(Segment {
                                id: T::parse1(name, &mut z.sequence),
                                sequence: SeqIndex::parse1(sequence, &mut z.sequence),
                                length: size,
                                opt: S::parse1(opt, &mut z.sequence),
                            });
                        } else {
                            let sequence = split_line.next().unwrap();
                            let size = split_line.next().unwrap().parse().unwrap();
                            let opt = split_line.next();

                            z.segments.push(Segment {
                                id: T::parse1(name, &mut z.sequence),
                                sequence: SeqIndex::parse1(sequence, &mut z.sequence),
                                length: size,
                                opt: S::parse1(opt, &mut z.sequence),
                            });
                        }
                    }
                    "H" => {
                        let header = Header::from_string(&line_string);
                        z.header = header;
                    }
                    "L" => {
                        let from = split_line.next().unwrap();
                        let from_dir = split_line.next().unwrap() == "+";
                        let to = split_line.next().unwrap();
                        let to_dir = split_line.next().unwrap() == "+";
                        let overlap = split_line.next();
                        let opt = split_line.next();
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
                        let name = split_line.next().unwrap().to_owned();
                        //let (dirs, node_id) = path_parser(split_line.next().unwrap(), &mut z.sequence);
                        let a = split_line.next().unwrap().split(',');
                        let (mut dirs, mut node_id) = (
                            Vec::with_capacity(a.clone().count()),
                            Vec::with_capacity(a.clone().count()),
                        );
                        for d in a {
                            dirs.push(&d[d.len() - 1..] == "+");
                            node_id.push(SampleType::parse1(&d[..d.len() - 1], &mut z.sequence));
                        }

                        let k = U::parse1(split_line.next(), &mut z.sequence);
                        let k2 = S::parse1(split_line.next(), &mut z.sequence);
                        z.paths.push(Path {
                            name,
                            dir: dirs,
                            nodes: node_id,
                            overlap: k,
                            opt: k2,
                        });
                    }
                    "W" => {
                        let sample_id = split_line.next().unwrap().to_owned();
                        let hap_index = split_line.next().unwrap().parse().unwrap();
                        let seq_id = split_line.next().unwrap().to_owned();
                        let seq_start = split_line.next().unwrap().parse().unwrap();
                        let seq_end = split_line.next().unwrap().parse().unwrap();
                        let (w1, w2) = walk_parser(split_line.next().unwrap(), &mut z.sequence);
                        let opt = S::parse1(split_line.next(), &mut z.sequence);
                        z.walk.push(Walk {
                            sample_id,
                            hap_index,
                            seq_id,
                            seq_start,
                            seq_end,
                            walk_dir: w1,
                            walk_id: w2,
                            opt: opt,
                        });
                    }
                    "C" => {
                        let container = split_line.next().unwrap();
                        let container_dir = split_line.next().unwrap() == "+";
                        let contained = split_line.next().unwrap();
                        let contained_dir = split_line.next().unwrap() == "+";
                        let pos = split_line.next().unwrap().parse().unwrap();
                        let overlap = split_line.next().unwrap();
                        let opt = split_line.next();
                        z.containment.push(Containment {
                            container: T::parse1(container, &mut z.sequence),
                            container_dir,
                            contained: T::parse1(contained, &mut z.sequence),
                            contained_dir,
                            pos,
                            overlap: SeqIndex::parse1(overlap, &mut z.sequence),
                            opt: S::parse1(opt, &mut z.sequence),
                        });
                    }
                    "J" => {
                        let from = split_line.next().unwrap();
                        let from_dir = split_line.next().unwrap() == "+";
                        let to = split_line.next().unwrap();
                        let to_dir = split_line.next().unwrap() == "+";
                        let distance = parse_dumb(split_line.next().unwrap());
                        let opt = split_line.next();
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
            z.segments.sort_by(|a, b| a.id.cmp(&b.id));
            z.is_digit = T::is_digit();
            z.index_of_index = z.segments.iter().enumerate().map(|x| x.0).collect();
            z.index_low = z.segments[0].id.get_usize();
            z
        } else {
            Gfa::new()
        }
    }

    /// Convert Walk to Path
    pub fn walk_to_path(&mut self, sep: &str) {

        for walk in self.walk.iter() {
            let f = walk.walk_id.iter().map(|x| x.clone()).collect();
            let o = U::parse1(None, &mut self.sequence);
            let n = walk.sample_id.to_owned()
                + sep
                + &walk.hap_index.to_owned().to_string()
                + sep
                + &walk.seq_id
                + ":"
                + &walk.seq_start.to_owned().to_string()
                + "-"
                + &walk.seq_end.to_owned().to_string();
            let w = walk.walk_dir.iter().map(|x| *x).collect();
            let opt =  walk.opt.clone();
            self.paths.push(Path {
                name: n,
                dir: w,
                nodes: f,
                overlap: o,
                opt: opt,
            });
        }
        self.walk = Vec::new();
    }

    /// Not 100%, but still okay
    ///
    /// Does not work with String and SeqIndex
    pub fn is_compact(&self) -> bool {
        self.segments[0].id == T::parse1("1", &mut String::new())
            && self.segments[self.segments.len() - 1].id
                == T::parse1(&self.segments.len().to_string(), &mut String::new())
    }

    // pub fn make_compact(&mut self) {
    //     let mut mmax = &self.segments[self.segments.len() - 1].id;
    //     let mut last = &self.segments[0].id;
    //     for x in self.segments.iter() {
    //         if last as usize != &x.id as usize - 1  {
    //             self.segments.push(Segment {
    //                 id: last + 1,
    //                 sequence: SeqIndex([0, 0]),
    //                 length: 0,
    //                 opt: T::parse1("0", &mut String::new()),
    //             });
    //         }
    //     }
    // }

    /// Get node by id
    pub fn get_node_by_id(&self, id: &T) -> &Segment<T, S> {
        if self.is_digit{
            &self.get_node_digit(&id.get_usize())
        } else {
            &self.segments[self.segments.binary_search_by(|x| x.id.cmp(id)).unwrap()]
        }
    }

    pub fn get_node_digit(&self, id: &usize) -> &Segment<T, S> {
        let index = self.index_of_index[*id]-self.index_low;
        &self.segments[index]
    }


}

impl Gfa<u32, (), ()> {
    pub fn get_ind(&self, id: u32) -> &Segment<u32, ()> {
        &self.segments[self.segments.binary_search_by(|x| x.id.cmp(&id)).unwrap()]
    }
}

/// Get the version of a GFA file
pub fn get_version(file_name: &str) -> f32 {
    let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
    let reader = BufReader::new(file);
    let mut version_number = 0.0;
    for line in reader.lines() {
        let l = line.unwrap();
        if l.starts_with('H') {
            let a = l.split_whitespace().nth(1).unwrap();
            version_number = a.split(':').nth(2).unwrap().parse().unwrap();
            break;
        }
    }
    version_number
}

/// Check if a gfa file only contains of numeric segments
pub fn check_numeric_gfafile(file_name: &str) -> bool {
    let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let l = line.unwrap();
        if l.starts_with('S') {
            let a = l.split_whitespace().nth(1).unwrap();
            if a.parse::<u64>().is_err() {
                return false;
            }
        }
    }
    true
}

/// Check if a gfa file only contains of numeric segments and is compact
pub fn check_numeric_compact_gfafile(file_name: &str) -> (bool, bool) {
    let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");
    let reader = BufReader::new(file);
    let mut p = Vec::new();
    for line in reader.lines() {
        let l = line.unwrap();
        if l.starts_with('S') {
            let a = l.split_whitespace().nth(1).unwrap();
            if a.parse::<u64>().is_err() {
                return (false, false);
            } else {
                p.push(a.parse::<u64>().unwrap());
            }
        }
    }
    p.sort();
    if p[0] == 1 && p[p.len() - 1] == p.len() as u64 {
        (true, true)
    } else {
        (true, false)
    }
}

#[inline]
/// Parse a path
///
/// Separate node and direction with a comma
fn path_parser<T: SampleType>(path: &str, s: &mut String) -> (Vec<bool>, Vec<T>) {
    let a = path.split(',');
    let (mut dirs, mut node_id) = (
        Vec::with_capacity(a.clone().count()),
        Vec::with_capacity(a.clone().count()),
    );
    for d in a {
        dirs.push(&d[d.len() - 1..] == "+");
        node_id.push(SampleType::parse1(&d[..d.len() - 1], s));
    }
    (dirs, node_id)
}

/// Parse a walk
fn walk_parser<T: SampleType>(walk: &str, s1: &mut String) -> (Vec<bool>, Vec<T>) {
    let a = walk[1..].split(['<', '>']).count();
    let (mut dirs, mut node_id) = (Vec::with_capacity(a), Vec::with_capacity(a));
    dirs.push(walk.starts_with('>'));
    let mut s = String::new();
    for x in walk[1..].chars() {
        if x == '<' || x == '>' {
            dirs.push(x == '>');
            node_id.push(T::parse1(&s, s1));
            s = String::new();
        } else {
            s.push(x);
        }
    }
    node_id.push(T::parse1(&s, s1));

    (dirs, node_id)
}

pub fn fill_nodes(graph: &mut Gfa<u32, (), ()>) {
    graph.segments.sort();

    let mut filled_vec = Vec::new();
    let mut prev_value = graph.segments[0].id;

    // Iterate through the sorted vector and find missing values
    for value in graph.segments.iter() {
        // Check if there are missing values between previous value and current value
        if value.id > prev_value + 1 {
            // Insert missing values
            for missing_value in prev_value + 1..value.id {
                filled_vec.push(Segment {
                    id: missing_value,
                    sequence: SeqIndex([0, 0]),
                    length: 0,
                    opt: (),
                });
            }
        }
        filled_vec.push(Segment {
            id: value.id,
            sequence: value.sequence.clone(),
            length: value.length,
            opt: (),
        });
        prev_value = value.id;
    }
    graph.segments = filled_vec;
}

/// Parse a string to a generic type
///
/// Only needed for Jumps
fn parse_dumb(s: &str) -> i64 {
    if s == "*" {
        -1
    } else {
        s.parse().unwrap()
    }
}

#[derive(Debug, Clone)]
/// PanSN-spec path data structure
/// PanSN-spec
/// [sample_name][delim][haplotype_id][delim][contig_or_scaffold_name]
pub struct Pansn<'a, T: SampleType, S: Opt, U: Opt> {
    pub genomes: Vec<Sample<'a, T, S, U>>,
}

#[derive(Debug, Clone)]
pub struct Sample<'a, T: SampleType, S: Opt, U: Opt> {
    pub name: String,
    pub haplotypes: Vec<Haplotype<'a, T, S, U>>,
}

#[derive(Debug, Clone)]
/// PanSN-spec haplotype
///
/// Merging multiple paths together
pub struct Haplotype<'a, T: SampleType, S: Opt, U: Opt> {
    pub name: String,
    pub paths: Vec<&'a Path<T, S, U>>,
}
impl<'a, T: SampleType, S: Opt, U: Opt> Default for Pansn<'a, T, S, U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: SampleType, S: Opt, U: Opt> Pansn<'a, T, S, U> {
    pub fn new() -> Self {
        Self {
            genomes: Vec::new(),
        }
    }

    /// Create Pansn from a list of paths
    /// ```
    /// ```
    pub fn from_graph(paths: &'a [Path<T, S, U>], del: &str) -> Self {
        let mut genomes: Vec<Sample<'a, T, S, U>> = Vec::new();

        // All path names
        let a: Vec<String> = paths.iter().map(|x| x.name.to_string()).collect();

        // Check if all path names are in Pansn-spec
        let b = a
            .iter()
            .map(|x| x.split(del).collect::<Vec<&str>>().len())
            .collect::<Vec<usize>>()
            .iter()
            .all(|&x| x == 3);

        // If no del -> one path is one haplotype, is one genome
        if del == " " || !b {
            for path in paths.iter() {
                genomes.push(Sample {
                    name: path.name.to_string(),
                    haplotypes: vec![Haplotype {
                        name: path.name.to_string(),
                        paths: vec![path],
                    }],
                })
            }
        } else {
            for path in paths.iter() {
                let name_split: Vec<&str> = path.name.split(del).collect();
                let genome;
                let haplotype;
                if name_split.len() > 1 {
                    genome = name_split[0].to_string();
                    haplotype = name_split[1].to_string();
                } else {
                    genomes.push(Sample {
                        name: path.name.to_string(),
                        haplotypes: vec![Haplotype {
                            name: path.name.to_string(),
                            paths: vec![path],
                        }],
                    });
                    panic!("No Pansn, remove sep or adjust gfa")
                }
                // Gibt es schon so ein Genome?
                if let Some((index1, _)) = genomes
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.name == genome)
                {
                    let genome = &mut genomes[index1];
                    // Gibt es schon ein Haplotype
                    if let Some((index2, _)) = genome
                        .haplotypes
                        .iter()
                        .enumerate()
                        .find(|(_, item)| item.name == haplotype)
                    {
                        let haplo = &mut genome.haplotypes[index2];
                        haplo.paths.push(path);
                    } else {
                        let haplo = Haplotype {
                            name: haplotype,
                            paths: vec![path],
                        };
                        genome.haplotypes.push(haplo);
                    }
                } else {
                    let haplo = Haplotype {
                        name: haplotype,
                        paths: vec![path],
                    };
                    let genome = Sample {
                        name: genome,
                        haplotypes: vec![haplo],
                    };
                    genomes.push(genome);
                }
            }
        }
        Pansn { genomes }
    }

    /// Get path for each haplotype
    pub fn get_haplo_path(&self) -> Vec<(String, Vec<&Path<T, S, U>>)> {
        let mut result = Vec::new();
        for x in self.genomes.iter() {
            for y in x.haplotypes.iter() {
                let kk: Vec<_> = y.paths.to_vec();
                result.push((x.name.clone() + "#" + &y.name, kk));
            }
        }

        result
    }

    /// Get path for each genome
    pub fn get_path_genome(&self) -> Vec<(String, Vec<&Path<T, S, U>>)> {
        let mut result = Vec::new();
        for x in self.genomes.iter() {
            let mut aa = Vec::new();
            for y in x.haplotypes.iter() {
                let kk: Vec<_> = y.paths.to_vec();
                aa.extend(kk);
            }
            result.push((x.name.clone(), aa));
        }

        result
    }

    /// Get all path
    pub fn get_paths_direct(&self) -> Vec<(String, Vec<&Path<T, S, U>>)> {
        let mut result = Vec::new();
        for x in self.genomes.iter() {
            for y in x.haplotypes.iter() {
                y.paths
                    .iter()
                    .for_each(|i| result.push((i.name.to_string(), vec![*i])))
            }
        }
        result
    }

    pub fn number_of_pansn(&self) {
        println!("Number of genomes: {}", self.get_path_genome().len());
        println!(
            "Number of individual haplotypes: {}",
            self.get_haplo_path().len()
        );
        println!("Total number of paths: {}", self.get_paths_direct().len());
    }
}
