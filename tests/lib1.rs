use gfa_reader::{Edge, Gfa, GraphWrapper, NCGfa, NCPath, Path};

#[test]
/// Check full header
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
fn read_gfa_edges() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, true);
    assert_eq!(&graph.edges.as_ref().unwrap().len(), &35381);
    assert_eq!(&graph.edges.clone().unwrap()[0],
               &Edge { from: "1".to_string(), from_dir: true, to_dir: true, to: "78".to_string(), overlap: "0M".to_string(), opt: () });

    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    assert_eq!(graph.edges, None);
}


#[test]
fn read_gfa_nodes() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    assert_eq!(graph.nodes[9].opt, ());
    assert_eq!(graph.nodes.len(), 26234);
    assert_eq!(graph.paths[0].nodes[0], "4".to_string());
}


#[test]
fn read_gfa_nodes2() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    let mut gra: GraphWrapper<Path> = GraphWrapper::new();
    gra.from_gfa(&graph.paths, " ");
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
    assert_eq!(graph.nodes[1-1].id, 1);
    if graph.mapper.len() != graph.nodes.len() {
        graph.mapper = graph.nodes.iter().map(| x| x.id.to_string()).collect();
    }
    assert_eq!(graph.get_old_node(&1),  &1.to_string());
    assert_eq!(graph.nodes.len(), 26234);
    assert_eq!(graph.nodes[0].id, 1);
    assert_eq!(graph.paths[0].nodes[0], 4);


}


#[test]
fn read_ncgfa_nodes() {
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: NCGfa<()> = NCGfa::new();
    graph.parse_gfa_file_direct(filename, false);
    assert_eq!(graph.nodes[9].opt, ());
    assert_eq!(graph.nodes[1-1].id, 1);
}

#[test]
fn convert_gfa_ncgfa(){
    eprintln!("Read gfa");
    // Example data
    let filename = "data/size5.gfa";
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, false);
    let a = graph.convert_to_ncgraph(&graph, true);
    assert_eq!(graph.nodes[1].seq, a.nodes[1].seq);
    assert_eq!(graph.nodes[1].opt, a.nodes[1].opt);
    assert_eq!(graph.nodes[1].id, a.nodes[1].id.to_string());
    assert_eq!(a.nodes[1].id, 2)
}






//
//
//
// #[test]
// fn read_gfa() {
//     eprintln!("Read gfa");
//     // Example data
//     let filename = "data/size5.gfa";
//     let mut graph: Gfa<()> = Gfa::new();
//     graph.parse_gfa_file(filename);
//     assert_eq!(graph.nodes[9].seq, "C");
//     assert_eq!(graph.nodes[8].opt, ());
//
//
// }
//
//
// #[test]
// fn read_ncgfa() {
//     let filename = "data/size5.gfa";
//     let mut graph: NCGfa = NCGfa::new();
//     graph.parse_gfa_file(filename);
//     assert_eq!(graph.nodes.get(8).unwrap().seq, "C");
//     assert_eq!(graph.nodes.get(8).unwrap().id, 8);
//     assert_eq!(graph.nodes.get(graph.nodes.len()-1).unwrap().id as usize, graph.nodes.len()-1);
//
// }
//
//
// fn read_gfa_convert_ncgfa() {
//     let filename = "data/size5.gfa";
//     let mut graph: Gfa<()> = Gfa::new();
//     graph.parse_gfa_file(filename);
//     let mut graph2: NCGfa = NCGfa::new();
//     let mapper = graph2.make_mapper(&mut graph);
//     graph2.convert_with_mapper(mapper, &graph);
//     assert_eq!(graph2.nodes.get(7).unwrap().seq, "C");
//     assert_eq!(graph2.nodes.get(8).unwrap().id, 8);
//     assert_eq!(graph2.nodes.get(graph2.nodes.len()-1).unwrap().id as usize, graph2.nodes.len()-1);
//
// }