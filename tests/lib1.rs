use gfa_reader::{Gfa, Link, NCGfa, Pansn, Path};

#[test]
/// Read GFA
/// -  nodes
/// - pansn
fn read_gfa_header() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    assert_eq!(graph.header.version_number, "1.0".to_string());
    assert_eq!(graph.header.tag, "VN".to_string());
    assert_eq!(graph.header.typ, "Z".to_string());
}

#[test]
/// Read GFA
/// - edges
/// - pansn
fn read_gfa_edges() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, true);
    assert_eq!(&graph.links.as_ref().unwrap().len(), &35381);
    assert_eq!(
        &graph.links.clone().unwrap()[0],
        &Link {
            from: "1".to_string(),
            from_dir: true,
            to_dir: true,
            to: "78".to_string(),
            overlap: "0M".to_string(),
            opt: ()
        }
    );

    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    assert_eq!(graph.links, None);
}

#[test]
fn read_gfa_nodes() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    assert_eq!(graph.segments[9].opt, ());
    assert_eq!(graph.segments.len(), 26234);
    assert_eq!(graph.paths[0].nodes[0], "4".to_string());
}

#[test]
fn read_gfa_nodes2() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    let gra: Pansn<Path> = Pansn::from_graph(&graph.paths, " ");
    assert_eq!(gra.genomes.len(), 5);
}

#[test]
fn read_gfa_convert_nodes() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: NCGfa<()> = NCGfa::new();
    graph.parse_gfa_file_direct(filename, false);
    assert_eq!(graph.nodes[9].opt, ());
    assert_eq!(graph.nodes[1 - 1].id, 1);
}

#[test]
fn read_gfa_convert_nodes2() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: NCGfa<()> = NCGfa::new();
    graph.parse_gfa_file_and_convert(filename, false);
    assert_eq!(graph.nodes[9].opt, ());
    assert_eq!(graph.nodes[1 - 1].id, 1);
    assert_eq!(graph.nodes.len(), 26234);
    assert_eq!(graph.nodes[0].id, 1);
    assert_eq!(graph.paths[0].nodes[0], 4);
    assert_eq!(graph.mapper.unwrap().len(), 26234);
}

#[test]
fn read_ncgfa_nodes() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: NCGfa<()> = NCGfa::new();
    graph.parse_gfa_file_direct(filename, false);
    assert_eq!(graph.nodes[9].opt, ());
    assert_eq!(graph.nodes[1 - 1].id, 1);
}

#[test]
fn convert_gfa_ncgfa() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    let a = graph.convert_to_ncgraph(&graph);
    assert_eq!(graph.segments[1].sequence, a.nodes[1].seq);
    assert_eq!(graph.segments[1].opt, a.nodes[1].opt);
    assert_eq!(graph.segments[1].name, a.nodes[1].id.to_string());
    assert_eq!(a.nodes[1].id, 2)
}

#[test]
fn convert_gfa_ncgfa2() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    let a = graph.convert_to_ncgraph(&graph);
    assert!(a.check_numeric());
}

#[test]
fn convert_gfa_ncgfa3() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: NCGfa<()> = NCGfa::new();
    graph.parse_gfa_file_direct(filename, false);
}
