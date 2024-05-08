use gfa_reader::{Gfa, Pansn, SeqIndex};

#[test]
/// Read GFA
/// -  nodes
/// - pansn
fn read_gfa_header() {
    let mut gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file("data/primates-pg.shuffle.gfa");
    gfa.walk_to_path();
    let _o = gfa.segments[0].sequence.get_string(&mut gfa.sequence);
    assert_eq!(_o, "TCTTTCTGGTGCAA");
    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.segments[0].id, 1);
    assert!(gfa.is_compact());


// No edges and no optional fields
    //let mut graph: Gfa<String, (), ()> = Gfa::parse_gfa_file("data/size5.gfa");

// No edges and no optional fields
    let mut graph: Gfa<SeqIndex, (), ()> = Gfa::parse_gfa_file("data/size5.gfa");

    let a = Pansn::from_graph(&graph.paths, "#");
    assert!(a.genomes.len() > 0);
}

#[test]
/// Read GFA
/// -  nodes
/// - pansn
fn read_gfa_header2() {
    let mut gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file("data/primates-pg.shuffle.gfa");
    gfa.walk_to_path();
    let o = gfa.get_node_by_id(1).id;
    assert_eq!(o, 1);
}


