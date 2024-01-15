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
        format!("H\tVN:Z:\t{}\n", self.version_number)
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
    pub tag: String,
    pub typee: String,
    pub value: String,
}

impl OptElem {
    /// Write optional field to string
    fn to_string1(&self) -> String{
        format!("{}\t{}\t{}", self.tag, self.typee, self.value)
    }
}


/// Trait for OptFields
pub trait OptFields: Sized + Default + Clone {

    /// Return a slice over all optional fields. NB: This may be
    /// replaced by an iterator or something else in the future
    fn fields(&self) -> &[OptElem];

    /// Given an iterator over a split, each expected to hold one
    /// optional field (in the <TAG>:<TYPE>:<VALUE> format), parse
    /// them as optional fields to create a collection.
    fn parse(input: Split<&str>) -> Self;

    fn new() -> Self;



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


}


/// Stores all the optional fields in a vector.
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
            fields.push(OptElem { tag: tag.to_string(), typee: typ.to_string(), value: val.to_string()});

        }
        fields
    }

    fn new() -> Self {
        Vec::new()
    }


}



#[derive(Debug)]
/// GFA segment line
///
/// Segment or nodes hold sequence
/// Feature:
///     - id
///     - seq
///     - op
///
/// Sequence are "optional", but are always represented in variation graphs
pub struct Segment<T: OptFields>{
    pub name: String,
    pub sequence: String,
    pub size: u32,
    pub opt: T,
}


impl <T: OptFields> Segment<T> {

    /// Write node to string
    fn to_string(&self) -> String {
        let a = format!("S\t{}\t{}\n", self.name, self.sequence.len());

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

        format!(">{}\n{}", self.name, self.sequence)
    }
}




#[derive(Debug, PartialEq, Clone, Default)]
/// GFA containment line
///
/// Fields:
/// - From
/// - From direction
/// - To
/// - To direction
/// - Overlap (Link + containment)
/// - Pos
/// - Ops
///
/// Comment:
/// Very similar to links
pub struct Containment<T: OptFields>{
    pub container: String,
    pub container_orient: bool,
    pub contained: String,
    pub contained_orient: bool,
    pub pos : usize, // Position of the overlap
    pub overlap: String,
    pub opt: T,
}

impl <T: OptFields>Containment<T> {

    #[allow(dead_code)]
    /// Write edge to string
    fn to_string_link(&self) -> String {
        let a = format!("L\t{}\t{}\t{}\t{}\t{}\n", self.container, {if self.container_orient {"+"} else {"-"}}, self.contained, {if self.contained_orient {"+"} else {"-"}}, self.overlap);
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
/// GFA link line
///
/// Fields:
/// - From
/// - From direction
/// - To
/// - To direction
/// - Overlap (Link + containment)
/// - Ops
///
/// Comment:
/// Representation of "edges"
pub struct Link<T: OptFields>{
    pub from: String,
    pub from_dir: bool,
    pub to: String,
    pub to_dir: bool,
    pub overlap: String,
    pub opt: T,
}



impl <T: OptFields> Link<T> {

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
/// GFA path line:
///
/// Fields:
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
/// Path features:
/// - names
/// - Directions of the nodes
/// - Node names
/// - Overlap
///
/// Comment: When there is not that many paths, the amount of memory for the overlap is not that much.
pub struct Walk{
    pub sample_id: String,
    pub hap_index: usize,
    pub seq_id: String,
    pub seq_start: usize,
    pub seq_end: usize,
    pub walk: String,
}

impl Walk {

        #[allow(dead_code)]
        /// Write path to string (GFA1 format)
        /// v1.1
        fn to_string(&self) -> String {
            let a = format!("W\t{}\t{}\t{}\t{}\t{}\t{}\n", self.sample_id, self.hap_index, self.seq_id, self.seq_start, self.seq_end, self.walk);
            a
        }
}





#[derive(Debug)]
pub struct Fragment<T: OptFields>{
    pub sample_id: String,
    pub external_ref: usize,
    pub seg_begin: usize,
    pub seg_end: usize,
    pub frag_begin: usize,
    pub frag_end: usize,
    pub alignment: String,
    pub opt: T,
}

impl <T: OptFields>Fragment<T>{

    #[allow(dead_code)]
    /// Write path to string (GFA1 format)
    /// v2
    fn to_string(&self) -> String {
        let a = format!("F\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", self.sample_id, self.external_ref, self.seg_begin, self.seg_end, self.frag_begin, self.frag_end, self.alignment);
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
/// Ordered and unordered groups
/// v2.0
pub struct Group {
    pub is_ordered: bool,
    pub name: String,
    pub nodes: Vec<String>,
    pub direction: Vec<bool>,
}

impl Group {

    /// Write group to string (GFA2 format)
    pub fn to_string2(&self) -> String{
        let mut a = format!("{}\t", {if self.is_ordered{"O".to_string()} else {"U".to_string()}});
        a = format!("{}\t{}", a, self.name);
        if self.is_ordered {
            let f1: Vec<String> = self.nodes.iter().zip(&self.direction).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
            let f2 = f1.join("\t");
            format!("{}\t{}\n", a, f2)
        } else {
            let f1: Vec<String> = self.nodes.iter().map(|n| format!("{}", n)).collect();
            let f2 = f1.join("\t");
            format!("{}\t{}\n", a, f2)
        }
    }

    /// Write group to string (GFA1 format)
    /// P line in GFA1
    pub fn to_string(&self) -> String {
        let mut a = format!("{}\t", "P");
        a = format!("{}\t{}", a, self.name);
        if self.is_ordered {
            let f1: Vec<String> = self.nodes.iter().zip(&self.direction).map(|n| format!("{}{}", n.0, {if *n.1{"+".to_string()} else {"-".to_string()}})).collect();
            let f2 = f1.join(",");
            format!("{}\t{}\n", a, f2)
        } else  {
            format!("{}\n", a)
        }
    }
}



#[derive(Debug)]
///
pub struct Gap<T: OptFields>{
    pub name: String,
    pub sid1: String,
    pub sid1_ref: bool,
    pub sid2: String,
    pub sid2_ref: bool,
    pub dist: usize,
    pub tag: T,
}


impl <T: OptFields> Gap<T> {

    #[allow(dead_code)]
    /// Write path to string (GFA2 format)
    fn to_string(&self) -> String {
        let a = format!("G\t{}\t{}{}\t{}{}\t{}\n", self.name, self.sid1, {if self.sid1_ref{"+"} else {"-"}}, self.sid2, {if self.sid2_ref{"+"} else {"-"}}, self.dist);
        if self.tag.fields().len() > 0 {
            let b: Vec<String> = self.tag.fields().iter().map(|a| a.to_string1()).collect();
            let c = b.join("\t");
            format!("{}{}\n", a, c)
        } else {
            a
        }
    }
    
}

#[derive(Debug)]
pub struct Jump<T: OptFields>{
    pub from: String,
    pub from_orient: bool,
    pub to: String,
    pub to_orient: bool,
    pub distance: String,
    pub opt: T,
}

impl <T: OptFields>Jump<T>  {

        #[allow(dead_code)]
        /// Write path to string (GFA1 format)
        /// v1.2
        fn to_string(&self) -> String {
            let a = format!("J\t{}\t{}\t{}\t{}\t{}\n", self.from, {if self.from_orient {"+"} else {"-"}}, self.to, {if self.to_orient {"+"} else {"-"}}, self.distance);
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
/// GFA edge line
///
/// Edges are connection between segments (v2)
/// More information can be found here: https://gfa-spec.github.io/GFA-spec/GFA2.html#:~:text=GFA2%20is%20a%20generalization%20of,giving%20rise%20to%20each%20sequence.
pub struct Edges<T: OptFields>{
    pub id: u32,
    pub source_name: String,
    pub sink_name: String,
    pub source_dir: bool,
    pub sink_dir: bool,
    pub source_begin: u32,
    pub source_end: u32,
    pub sink_begin: u32,
    pub sink_end: u32,
    pub ends: u8, // Bits 1 = source_begin, 2 = source_end, 4 = sink_begin, 8 = sink_end
    pub alignment: String,
    pub opts: T
}

impl <T: OptFields>Edges<T>{

    #[allow(dead_code)]
    /// Edges to string (GFA2 format)
    fn to_string2(&self) -> String {
        let mut a = format!("E\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        self.id,
                        self.source_name,
                        {if self.source_dir{"+"} else {"-"}},
                        self.sink_name,
                        {if self.sink_dir{"+"} else {"-"}},
                        self.source_begin);
        if &self.ends & &1 != 0 {
            a.push('$');
        }
        use std::fmt::Write;

        write!(&mut a, "\t{}", self.source_end).unwrap();
        if self.ends & 2 != 0 {
            a.push('$');
        }

        write!(&mut a, "\t{}", self.sink_begin).unwrap();
        if self.ends & 4 != 0 {
            a.push('$');
        }

        write!(&mut a, "\t{}", self.sink_end).unwrap();
        if self.ends & 8 != 0 {
            a.push('$');
        }

        write!(&mut a, "\t{}", self.alignment).unwrap();

        if self.opts.fields().len() > 0 {
            let b: Vec<String> = self.opts.fields().iter().map(|a| a.to_string1()).collect();
            let c = b.join("\t");
            format!("{}{}\n", a, c)
        } else {
            a
        }
    }

}









#[derive(Debug)]
/// A representation of a GFA file (v1 + v2)
///
/// GFA v1 + 1.1/1.2 + v2
///
/// Comment: This implementation should be able to parse any kind of GFA file, but has increased
/// memory consumption, since most node ids are stored at Strings which are a minimum of 24 bytes.
/// This is only maintained, since it is not of further use in any of my projects.
pub struct Gfa<T: OptFields>{

    // GFA 1.0 data
    pub header: Header,
    pub segments: Vec<Segment<T>>,
    pub paths: Vec<Path>,
    pub links: Option<Vec<Link<T>>>,
    pub containments: Vec<Containment<T>>,

    // GFA 1.1/1.2 data
    pub walk: Vec<Walk>,
    pub jumps: Vec<Jump<T>>,

    // GFA 2.0 data
    pub edges: Vec<Edges<T>>,
    pub fragments: Vec<Fragment<T>>,
    pub groups: Vec<Group>,
    pub gaps: Vec<Gap<T>>,
    pub string2index: HashMap<String, usize>,
}



impl <T: OptFields> Gfa<T>{
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
            segments: Vec::new(),
            paths: Vec::new(),
            links: None,
            header: Header{tag: "".to_string(), typ: "".to_string(), version_number: "".to_string()},
            containments: Vec::new(),
            walk: Vec::new(), // v1.1
            jumps: Vec::new(), // v1.2
            string2index: HashMap::new(),

            edges: Vec::new(), // v2.0
            fragments: Vec::new(), // v2.0
            groups: Vec::new(), // v2.0
            gaps: Vec::new(), // 2.0

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
        if self.segments.len() == 0 {
            return None
        }


        // Check if the graph is numeric

        let is_digit = self.segments.iter().map(|x| x.name.chars().map(|g| g.is_ascii_digit()).collect::<Vec<bool>>().contains(&false)).collect::<Vec<bool>>().contains(&false);

        // Check if the numeric nodes are compact
        if is_digit {
            let mut numeric_nodes = self.segments.iter().map(|x| x.name.parse::<usize>().unwrap()).collect::<Vec<usize>>();
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
    pub fn parse_gfa_file(&mut self, file_name: &str, edges: bool) {


        if file_path::new(file_name).exists() {
            let file = File::open(file_name).expect("ERROR: CAN NOT READ FILE\n");

            // Parse plain text or gzipped file
            let reader: Box<dyn BufRead> = if file_name.ends_with(".gz") {
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };
            let version_number = get_version(file_name);


            let mut nodes: Vec<Segment<T>> = Vec::new();
            let mut links: Vec<Link<T>> = Vec::new();
            // Iterate over lines
            for line in reader.lines() {
                let l = line.unwrap();
                let l2 = l.clone();
                let mut a = l2.split("\t");
                let first = a.next().unwrap();
                let line_split: Vec<&str> = l.split("\t").collect();
                match first {
                    "S" => {
                        let name = a.next().unwrap().parse().unwrap();
                        if version_number < 2.0 {
                            let sequence: String = a.next().unwrap().parse().unwrap();
                            let size = sequence.len() as u32;
                            nodes.push(Segment { name, sequence, size, opt: T::parse(a) });
                        } else {
                            let sequence: String = a.next().unwrap().parse().unwrap();
                            let size = a.next().unwrap().parse().unwrap();
                            nodes.push(Segment { name, sequence, size, opt: T::parse(a) });
                        }



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
                        if edges {

                            //edges.push(Edge{from: line_split[1].parse().unwrap(), from_dir: if line_split[2] == "+" { !false } else { !true }, to: line_split[3].parse().unwrap(), to_dir: if line_split[4] == "+" { !false } else { !true }, overlap: line_split[5].parse().unwrap(), opt: T::parse(line_split)});
                            links.push(Link {from: a.next().unwrap().to_string(), from_dir: if a.next().unwrap() == "+" { !false } else { !true }, to: a.next().unwrap().to_string(), to_dir: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a)});

                        }

                    }
                    "C" => {
                        if edges {
                            self.containments.push(Containment {container: a.next().unwrap().to_string(), container_orient: if a.next().unwrap() == "+" { !false } else { !true }, contained: a.next().unwrap().to_string(), contained_orient: if a.next().unwrap() == "+" { !false } else { !true }, overlap: a.next().unwrap().to_string(), opt: T::parse(a), pos: 0 });

                        }
                    }
                    "H" => {
                        let header = Header::from_string(&l);
                        self.header = header;
                    }
                    "W" => {
                        let sample_id = a.next().unwrap().to_string();
                        let hap_index = a.next().unwrap().parse().unwrap();
                        let seq_id = a.next().unwrap().to_string();
                        let seq_start = a.next().unwrap().parse().unwrap();
                        let seq_end = a.next().unwrap().parse().unwrap();
                        let walk = a.next().unwrap().to_string();
                        self.walk.push(Walk{ sample_id, hap_index, seq_id, seq_start, seq_end, walk});
                    }
                    "J" => {
                        let from = a.next().unwrap().to_string();
                        let from_orient = if a.next().unwrap() == "+" { !false } else { !true };
                        let to = a.next().unwrap().to_string();
                        let to_orient = if a.next().unwrap() == "+" { !false } else { !true };
                        let distance = a.next().unwrap().to_string();
                        self.jumps.push(Jump{from, from_orient: from_orient, to, to_orient: to_orient, distance, opt: T::parse(a)});
                    }

                    "G" => {
                        let name = a.next().unwrap().to_string();
                        let sid1 = a.next().unwrap().to_string();
                        let sid2 = a.next().unwrap().to_string();
                        let dist = a.next().unwrap().parse().unwrap();
                        self.gaps.push(Gap{name, sid1, sid1_ref: false, sid2, sid2_ref: false, dist, tag: T::parse(a)});
                    }

                    "F" => {
                        let sample_id = a.next().unwrap().to_string();
                        let external_ref = a.next().unwrap().parse().unwrap();
                        let seg_begin = a.next().unwrap().parse().unwrap();
                        let seg_end = a.next().unwrap().parse().unwrap();
                        let frag_begin = a.next().unwrap().parse().unwrap();
                        let frag_end = a.next().unwrap().parse().unwrap();
                        let alignment = a.next().unwrap().to_string();
                        self.fragments.push(Fragment{sample_id, external_ref, seg_begin, seg_end, frag_begin, frag_end, alignment, opt: T::parse(a)});
                    }
                    "E" => {
                        let id = a.next().unwrap().parse().unwrap();

                        let (source_name, source_dir) = split_string(a.next().unwrap()).unwrap();
                        let (sink_name, sink_dir) = split_string(a.next().unwrap()).unwrap();

                        let mut end = 0;
                        let source_begin: String = a.next().unwrap().parse().unwrap();
                        end = if source_begin.ends_with("$"){end & 1} else {end};
                        let s1 = source_begin.replace("$", "").parse().unwrap();

                        let  s2: String = a.next().unwrap().parse().unwrap();
                        end = if s2.ends_with("$"){end & 2} else {end};
                        let s2 = s2.replace("$", "").parse().unwrap();

                        let  s3: String = a.next().unwrap().parse().unwrap();
                        end = if s3.ends_with("$"){end & 4} else {end};
                        let s3 = s3.replace("$", "").parse().unwrap();

                        let s4: String = a.next().unwrap().parse().unwrap();
                        end = if s4.ends_with("$"){end & 8} else {end};
                        let s4 = s4.replace("$", "").parse().unwrap();

                        let alignment = a.next().unwrap().to_string();

                        self.edges.push(Edges{id,
                            source_name: source_name.to_string(),
                            sink_name: sink_name.to_string(),
                            source_dir: source_dir,
                            sink_dir: sink_dir,
                            source_begin: s1,
                            source_end: s2,
                            sink_begin: s3,
                            sink_end: s4,
                            ends: end,
                            alignment: alignment,
                            opts: T::parse(a)});
                    }
                    "O" => {
                        let is_ordered = true;
                        let name = a.next().unwrap().to_string();
                        let nodes: Vec<(&str, bool)> = a.next().unwrap().split(" ").map(|d| split_string(d).unwrap()).collect();
                        let (nodes, direction): (Vec<&str>, Vec<bool>) = nodes.iter().cloned().unzip();
                        self.groups.push(Group{is_ordered, direction: direction, nodes: nodes.iter().map(|a| a.to_string()).collect(), name: name});
                    }
                    "U" => {
                        let is_ordered = false;
                        let name = a.next().unwrap().to_string();
                        let nodes: Vec<(&str, bool)> = a.next().unwrap().split(" ").map(|d| split_string(d).unwrap()).collect();
                        let (nodes, direction): (Vec<&str>, Vec<bool>) = nodes.iter().cloned().unzip();
                        self.groups.push(Group{is_ordered, direction: direction, nodes: nodes.iter().map(|a| a.to_string()).collect(), name: name});
                    }





                    _ => {
                    }
                }

            }
            if edges {
                self.links = Some(links);
            }
            self.segments.extend(nodes);

        }

    }

    /// Write the graph to a file
    pub fn to_file(self, file_name: &str){
        let f = File::create(file_name).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(f, "{}",  self.header.to_string1()).expect("Not able to write");
        for node in self.segments.iter() {
            write!(f, "{}", node.to_string()).expect("Not able to write");
        }
        match &self.links {
            Some(value) =>{
                for edge in value.iter() {
                    write!(f, "{}", edge.to_string_link()).expect("Not able to write");
                }
            }
            _ => {}
        }
        for path in self.paths.iter() {
            write!(f, "{}", path.to_string()).expect("Not able to write");
        }
    }



    pub fn convert_to_ncgraph(& self, graph: &Gfa<T>) -> NCGfa<T>{
        let mut ncgraph: NCGfa<T> = NCGfa::new();
        let f = ncgraph.make_mapper(graph);
        ncgraph.convert_with_mapper(f, &graph);
        ncgraph
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
            edges: Option::None,
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
            self.edges = Some(edges);

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
        let ncgraph: NCGfa<T> = graph.convert_to_ncgraph(&graph);
        self.header = ncgraph.header;
        self.nodes = ncgraph.nodes;
        self.edges = ncgraph.edges;
        self.paths = ncgraph.paths;
        self.mapper = ncgraph.mapper;

    }


    /// Creat a map from string node id -> numeric node id
    pub fn make_mapper(&mut self, graph: & Gfa<T>) -> HashMap<String, usize> {
        let mut f = graph.segments.iter().map(|x| x.name.clone()).collect::<Vec<String>>();
        f.sort_by_key(|digit| digit.parse::<u32>().unwrap());
        let mut wrapper = HashMap::new();
        for (i, node) in f.iter().enumerate() {
            wrapper.insert(node.clone(), i+1);
        }
        wrapper
    }

    /// Convert the "old" graph with the mapper
    ///
    /// Using the mapper from "make_mapper"
    pub fn convert_with_mapper(&mut self, mapper: HashMap<String, usize>, graph: &Gfa<T>){
        let mut nodes: Vec<NCNode<T>> = graph.segments.iter().map(|x| NCNode{id: mapper.get(&x.name).unwrap().clone() as u32, seq: x.sequence.clone(), opt: x.opt.clone()}).collect();
        nodes.sort_by_key(|a| a.id);
        self.nodes = nodes;
        self.edges = None;
        match &graph.links {
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

    /// Get original (string) node
    pub fn get_old_node(&self, node_id: &usize) -> &String{
        &self.mapper[node_id-1]
    }

    /// Write the graph to a file
    pub fn to_file(self, file_name: &str){
        let f = File::create(file_name).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(f, "{}",  self.header.to_string1()).expect("Not able to write");
        for node in self.nodes.iter() {
            write!(f, "{}", node.to_string()).expect("Not able to write");
        }
        match &self.edges {
            Some(value) =>{
                for edge in value.iter() {
                    write!(f, "{}", edge.to_string_link()).expect("Not able to write");
                }
            }
            _ => {}
        }
        for path in self.paths.iter() {
            write!(f, "{}", path.to_string2()).expect("Not able to write");
        }
    }

    /// Check if the graph is really numeric
    pub fn check_numeric(&self) -> bool{
        for (i, x) in self.mapper.iter().enumerate(){
            if (i+1).to_string() != *x{
                return false
            }
        }
        return true
    }

    /// Remove the mapper if not needed
    pub fn remove_mapper(&mut self){
        if self.check_numeric(){
            self.mapper = Vec::new();
        }
    }


}

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

fn get_version(file_path: &str) -> f32{
    let file = File::open(file_path).expect("ERROR: CAN NOT READ FILE\n");

    // Parse plain text or gzipped file
    let reader: Box<dyn BufRead> = if file_path.ends_with(".gz") {
        Box::new(BufReader::new(GzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };


    // Read the first line of the file
    let first_line  = reader.lines().next().unwrap().unwrap();
    let line = first_line.split("\t").nth(1).unwrap();
    let version_number = line.split(':').nth(2).unwrap().to_string();
    return version_number.parse::<f32>().unwrap();

}

fn split_string(input_string: &str) -> Option<(&str, bool)> {
    let len = input_string.len();

    if len >= 1 {
        let first_substring = &input_string[0..len - 1];
        let last_letter = if &input_string[len - 1..] == "+"{true} else {false};

        Some((first_substring, last_letter))
    } else {
        None
    }
}



//-----------------------------------------------------------------------------------------------------------------------------------------------------------------------

/// IsPath trait
///
/// Can be used to create a Pansn from a Gfa using NCPath or Path
pub trait IsPath {
    fn get_name(&self) -> &String;
}



impl IsPath for Path{
    fn get_name(&self) -> &String{
        &self.name
    }
}

impl IsPath for NCPath{
    fn get_name(&self) -> &String{
        &self.name
    }
}


/// PanSN-spec path data structure
///
/// PanSN-spec
/// [sample_name][delim][haplotype_id][delim][contig_or_scaffold_name]
pub struct Pansn<'a, T: IsPath>{
    pub genomes: Vec<Sample<'a, T>>,
}

pub struct Sample<'a, T: IsPath>{
    pub name: String,
    pub haplotypes: Vec<Haplotype<'a, T>>

}

/// PanSN-spec haplotype
///
/// Merging multiple paths together
pub struct Haplotype<'a, T: IsPath> {
    pub name: String,
    pub paths: Vec<&'a T>
}

impl <'a, T: IsPath> Pansn<'a, T> {

    /// Create Pansn from a list of paths
    pub fn from_graph(paths: &'a Vec<T>, del: &str) -> Self{
        let mut genomes: Vec<Sample<'a, T>> = Vec::new();

        // If no del -> one path is one haplotype is one path
        if del == " " {
            for path in paths.iter() {
                genomes.push(Sample {name: path.get_name().to_string(), haplotypes: vec![Haplotype{name: path.get_name().to_string(), paths: vec![path]}]})
            }
        } else {
            for path in paths.iter() {
                let name_split: Vec<&str> = path.get_name().split(del).collect();
                let genome;
                let haplotype;
                if name_split.len() > 1{
                    genome = name_split[0].to_string();
                    haplotype = name_split[1].to_string();
                } else {
                    panic!("No Pansn, remove sep or adjust gfa")
                }
                // Gibt es schon so ein Genome?
                if let Some((index1, _)) = genomes.iter().enumerate().find(|(_, item)| item.name == genome) {
                    let genome = &mut genomes[index1];
                    // Gibt es schon ein Haplotype
                    if let Some((index2, _)) = genome.haplotypes.iter().enumerate().find(|(_, item)| item.name == haplotype) {
                        let haplo = &mut genome.haplotypes[index2];
                        haplo.paths.push(path);
                    } else {
                        let haplo = Haplotype{name: haplotype, paths: vec![path]};
                        genome.haplotypes.push(haplo);

                    }
                } else {
                    let haplo = Haplotype{name: haplotype, paths: vec![path]};
                    let genome = Sample {name: genome, haplotypes: vec![haplo]};
                    genomes.push(genome);
                    println!("Did not find the specific string.");
                }

            }
        }
        Pansn {
            genomes,
        }
    }

    pub fn get_haplo_path(&self) -> Vec<(String, Vec<&'a T>)>{
        let mut result = Vec::new();
        for x in self.genomes.iter(){
            for y in x.haplotypes.iter(){
                let kk: Vec<&T> = y.paths.iter().map(|i| *i).collect();
                result.push((x.name.clone() + "#" + &y.name, kk));
            }
        }

        result
    }

    pub fn get_path_genome(&self) -> Vec<(String, Vec<&'a T>)>{
        let mut result = Vec::new();
        for x in self.genomes.iter(){
            let mut aa = Vec::new();
            for y in x.haplotypes.iter(){
                let kk: Vec<&T> = y.paths.iter().map(|i| *i).collect();
                aa.extend(kk);
            }
            result.push((x.name.clone(), aa));
        }

        result
    }

    pub fn get_paths_direct(&self) -> Vec<(String, Vec<&'a T>)>{
        let mut result = Vec::new();
        for x in self.genomes.iter(){
            for y in x.haplotypes.iter(){
                y.paths.iter().for_each(|i| result.push((i.get_name().to_string(), vec![*i])))
            }
        }
        return result
    }

}

