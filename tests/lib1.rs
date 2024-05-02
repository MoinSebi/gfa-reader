use gfa_reader::Gfa;

#[test]
/// Read GFA
/// -  nodes
/// - pansn
fn read_gfa_header() {
    let mut gfa_: Gfa<u64, (), ()> = Gfa::parse_gfa_file("data/primates-pg.shuffle.gfa");
    gfa_.walk_to_path();
    assert_eq!(gfa_.walk.len(), 0);
    assert_eq!(gfa_.segments[0].id, 1);
    assert_eq!(gfa_.segments[1].id, 2);
    assert_eq!(gfa_.is_compact(), true);

}
