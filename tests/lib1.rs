use gfa_reader::{check_numeric_compact_gfafile, check_numeric_gfafile, Gfa, SeqIndex};

#[test]
/// Read GFA
///
/// + check header
fn read_gfa_header() {
    let mut gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    gfa.walk_to_path("#");
    let _o = gfa.segments[0].sequence.get_string(&gfa.sequence);
    assert_eq!(_o, "AAAAAAAAAA");
    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.segments[0].id, 1);
}

#[test]
/// Read GFA
///
/// + check header
fn read_gfa_string() {
    let mut gfa: Gfa<SeqIndex, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    gfa.walk_to_path("#");
    let _o = gfa.segments[0].sequence.get_string(&gfa.sequence);
    assert_eq!(_o, "AAAAAAAAAA");
    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.get_node_by_id(&gfa.segments[0].id).length, 10);
}

#[test]
/// Read GFA
///
/// + check header
fn read_gfa_string_vs_multi() {
    let mut gfa: Gfa<SeqIndex, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    let mut gfa2: Gfa<SeqIndex, (), ()> =
        Gfa::parse_gfa_file_multi("data/testGraph_complex.gfa", 2);

    gfa.walk_to_path("#");
    gfa2.walk_to_path("#");

    assert_eq!(
        gfa.segments[1].sequence.get_string(&gfa.sequence),
        gfa2.segments[1].sequence.get_string(&gfa.sequence)
    );

    assert_eq!(gfa.segments.len(), gfa2.segments.len());

    assert_eq!(gfa.links.len(), gfa2.links.len());

    assert_eq!(
        gfa.segments[0].sequence.get_string(&gfa.sequence),
        gfa2.segments[0].sequence.get_string(&gfa.sequence)
    );

    assert_eq!(gfa.paths[0].dir, gfa2.paths[0].dir);

    assert_eq!(gfa.walk.len(), gfa2.walk.len());
    assert_eq!(
        gfa.get_node_by_id(&gfa.segments[0].id).length,
        gfa2.get_node_by_id(&gfa.segments[0].id).length
    );
}

#[test]
/// Read GFA
///
/// + check header
fn read_gfa_get_sequence() {
    let mut gfa: Gfa<SeqIndex, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    gfa.walk_to_path("#");
    let _o = gfa.segments[0].sequence.get_string(&gfa.sequence);
    assert_eq!(_o, gfa.get_sequence_by_id(&gfa.segments[0].id));
    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.get_node_by_id(&gfa.segments[0].id).length, 10);
}

#[test]
/// Read GFA
///
/// + check header
fn read_gfa_get_sequence_digit() {
    let mut gfa: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    gfa.walk_to_path("#");
    let _o = gfa.segments[0].sequence.get_string(&gfa.sequence);
    assert_eq!(_o, gfa.get_sequence_by_id(&gfa.segments[0].id));
    assert_eq!(_o, gfa.get_sequence_by_digit(&gfa.segments[0].id));

    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.get_node_by_id(&gfa.segments[0].id).length, 10);
}



#[test]
/// READ GFA 1.1
fn read_gfa_header2() {
    let mut gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file("data/testGraph_1.1.gfa");
    assert_eq!(gfa.walk[gfa.walk.len() - 1].walk_id.len(), 1);

    gfa.walk_to_path("#");
    let o = gfa.get_node_by_id(&1).id;
    //assert_eq!(gfa.paths.len(), 1);
    assert_eq!(gfa.walk.len(), 0);
    assert_eq!(gfa.paths.len(), 7);

    assert_eq!(gfa.paths[gfa.paths.len() - 1].nodes.len(), 1);
    assert_eq!(o, 1);
}

#[test]
/// Check numeric (external)
fn check_numeric() {
    let gfa = check_numeric_gfafile("data/testGraph_complex.gfa");
    assert!(gfa);
    let gfa = check_numeric_gfafile("data/testGraph_1.1.gfa");
    assert!(gfa);
    let gfa = check_numeric_gfafile("data/testGraph_compact.gfa");
    assert!(gfa);
    let gfa = check_numeric_gfafile("data/testGraph_compact_nopw.gfa");
    assert!(gfa);
    let gfa = check_numeric_gfafile("data/testGraph_non-num.gfa");
    assert!(!gfa);
}

#[test]
/// Read GFA
/// -  nodes
/// - pansn
fn check_numeric2() {
    let gfa = check_numeric_compact_gfafile("data/testGraph_complex.gfa");
    assert_eq!(gfa, (true, false));
    let gfa = check_numeric_compact_gfafile("data/testGraph_non-num.gfa");
    assert_eq!(gfa, (false, false));
    let gfa = check_numeric_compact_gfafile("data/testGraph_compact.gfa");
    assert_eq!(gfa, (true, true));
    let gfa = check_numeric_compact_gfafile("data/testGraph_compact_nopw.gfa");
    assert_eq!(gfa, (true, true));
    let gfa = check_numeric_compact_gfafile("data/testGraph_1.1.gfa");
    assert_eq!(gfa, (true, true));

    let p: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/testGraph_compact.gfa");
    let p = p.is_compact();
    assert!(p);
    let p: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/testGraph_complex.gfa");
    let p = p.is_compact();
    assert!(!p);
    let p: Gfa<u32, (), ()> = Gfa::parse_gfa_file("data/testGraph_1.1.gfa");
    let p = p.is_compact();
    assert!(p);
}
