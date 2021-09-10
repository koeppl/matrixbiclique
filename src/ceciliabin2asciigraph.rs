extern crate env_logger;

#[macro_use] extern crate clap;

extern crate log;
use log::info;

use std::io::prelude::*;
use std::io::BufRead;
use byteorder::{LittleEndian, ReadBytesExt};

#[macro_use] extern crate more_asserts;


#[allow(dead_code)] mod common;

fn main() {
    let matches = clap_app!(count_r =>
        (about: "computes an ASCIIGraph from Cecilia's bin-format plus cliques in text format")
        (@arg input:  -i --infile  +takes_value "the input file to read (otherwise read from stdin)")
        (@arg clique:  -c --cliquefile  +takes_value "the clique file")
        (@arg output:  -o --outfile  +takes_value "filename for the binary output. If no output is given, it computes the page rank as explained in the paper.")
    ).get_matches();

    let mut nodes_reader = common::stream_or_stdin(matches.value_of("input"));


    let mut node2biclique = std::collections::HashMap::new(); //@ stores for each node ID on the left side of a biclique the index in the biclique array storing all nodes on the right side

    let mut bicliques = Vec::<Vec<u32>>::new();

    match matches.value_of("clique") {
        None => (),
        Some(cliquefilename) => {
            match std::fs::OpenOptions::new().write(false).read(true).create(false).open(cliquefilename) {
                Err(_) => (),
                Ok(file_handle) =>  {
                    let reader = std::io::BufReader::new(file_handle);
                    for line in reader.lines() {
                        let parsed_line = line.unwrap();
                        let splittedline = parsed_line.split('-').collect::<Vec<&str>>();
                        assert_eq!(splittedline.len(), 2);

                        bicliques.push(
                            splittedline[1].split(' ').map(|x| -> Option<u32> { 
                                let y = x.trim();
                                if y.len() > 0  {
                                    let i = y.parse::<u32>().unwrap();
                                    assert_gt!(i, 0);
                                    Some(i-1)
                                } else {
                                    None
                                }
                            }).filter(|x| x.is_some()).map(|x| x.unwrap()).collect());
                        for x in splittedline[0].split(' ') {
                            let y = x.trim();
                            if y.len() > 0  {
                                let key = y.parse::<u32>().unwrap();
                                assert_gt!(key, 0);
                                node2biclique.insert(key-1, bicliques.len()-1);
                            } 
                        }
                    }
                }
            }
        }
    };
    let max_biclique_node_id = node2biclique.keys().fold(std::u32::MIN, |a,b| a.max(*b)) as usize;

    let mut writer = common::stream_or_stdout(matches.value_of("output"));

    //common::stream_or_stdout(None);


    let num_nodes = nodes_reader.read_u32::<LittleEndian>().unwrap() as usize;
    let num_edges = nodes_reader.read_u32::<LittleEndian>().unwrap();
    info!("num_nodes : {}", num_nodes);
    info!("num_edges : {}", num_edges);


    let mut adjacency_matrix = vec![Vec::with_capacity(1); num_nodes];
       // Vec::with_capacity(num_nodes);
    let mut current_node = 0;

    let mut edge_counter = 0;
    loop {
        match nodes_reader.read_i32::<LittleEndian>() {
            Err(_) => break,
            Ok(field) => {
                if field < 0 { 
                    current_node = -field-1;
                    assert_lt!(current_node as usize, adjacency_matrix.len());
                    continue;
                }
                assert_gt!(field, 0);
                adjacency_matrix[current_node as usize].push(field as u32 - 1);
                edge_counter += 1;
            }
        }
    }
    assert_eq!(edge_counter, num_edges);

    info!("length of adjacency list: {}", adjacency_matrix.len());
    info!("maximum biclique node id: {}", max_biclique_node_id);

    writeln!(writer, "{}", std::cmp::max(num_nodes,max_biclique_node_id+1)).unwrap();
    for row_id in 0..adjacency_matrix.len() {
        let row = {
            if node2biclique.contains_key(&(row_id as u32)) {
                let mut row = adjacency_matrix[row_id].clone();
                row.append(&mut (bicliques[node2biclique[&(row_id as u32)] as usize].clone()));
                row.retain(|&x| x != row_id as u32);
                row.sort();
                row
            } else {
                adjacency_matrix[row_id].clone()
            }
        };
        writer.write(row.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ").as_bytes()).unwrap();
        if row.len() > 0 {
            writer.write(" ".as_bytes()).unwrap();
        }
        writer.write("\n".as_bytes()).unwrap();
    }
    for row_id in adjacency_matrix.len()..max_biclique_node_id+1 {
        let mut row = bicliques[node2biclique[&(row_id as u32)] as usize].clone();
        row.retain(|&x| x != row_id as u32);
        writer.write(row.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ").as_bytes()).unwrap();
        if row.len() > 0 {
            writer.write(" ".as_bytes()).unwrap();
        }
        writer.write("\n".as_bytes()).unwrap();
    }
    writer.flush().unwrap();

}
