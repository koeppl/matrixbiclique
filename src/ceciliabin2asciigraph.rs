extern crate env_logger;

#[macro_use] extern crate clap;

extern crate log;
use log::info;

use std::io::prelude::*;
use std::io::BufRead;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

#[macro_use] extern crate more_asserts;


#[allow(dead_code)] mod common;


fn spacelist2intarray(line: &str) -> Vec<u32> {
    let mut ret = Vec::new();
    for x in line.split(' ') {
        let y = x.trim();
        if y.len() > 0  {
            let i = y.parse::<u32>().unwrap();
            assert_gt!(i, 0);
            ret.push(i-1);
        }
    }
    ret
}

fn main() {
    let matches = clap_app!(count_r =>
        (about: "computes an ASCIIGraph from Cecilia's bin-format plus cliques in text format")
        (@arg input:  -i --infile  +takes_value "the input file to read (otherwise read from stdin)")
        (@arg clique:  -c --cliquefile  +takes_value "the clique file")
        (@arg output:  -o --outfile  +takes_value "filename for the binary output")
    ).get_matches();

    let mut nodes_reader = common::stream_or_stdin(matches.value_of("input"));
    let input_filename = match matches.value_of("input") {
        None => "stdin",
        Some(filename) => filename
    };
    let clique_filename = match matches.value_of("clique") {
        None => "none",
        Some(filename) => filename
    };

    use std::time::Instant;
    let now = Instant::now();

    let mut from_bicliques = Vec::<Vec<u32>>::new();
    let mut to_bicliques = Vec::<Vec<u32>>::new();


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

                        from_bicliques.push(spacelist2intarray(splittedline[0]));
                        to_bicliques.push(spacelist2intarray(splittedline[1]));
                        //
                        // bicliques.push(
                        //     splittedline[1].split(' ').map(|x| -> Option<u32> { 
                        //         let y = x.trim();
                        //         if y.len() > 0  {
                        //             let i = y.parse::<u32>().unwrap();
                        //             assert_gt!(i, 0);
                        //             Some(i-1)
                        //         } else {
                        //             None
                        //         }
                        //     }).filter(|x| x.is_some()).map(|x| x.unwrap()).collect());
                        // for x in splittedline[0].split(' ') {
                        //     let y = x.trim();
                        //     if y.len() > 0  {
                        //         let key = y.parse::<u32>().unwrap();
                        //         assert_gt!(key, 0);
                        //         node2biclique.insert(key-1, bicliques.len()-1);
                        //     } 
                        // }
                    }
                }
            }
        }
    };
    assert_eq!(to_bicliques.len(), from_bicliques.len());
    // let max_biclique_from_node = node2biclique.keys().fold(std::u32::MIN, |a,b| a.max(*b)) as usize;
    let max_biclique_from_node = from_bicliques.iter().map(|x| x.iter().fold(std::u32::MIN, |a,b| a.max(*b))).fold(std::u32::MIN, |a,b| a.max(b)) as usize;
    let max_biclique_to_node = to_bicliques.iter().map(|x| x.iter().fold(std::u32::MIN, |a,b| a.max(*b))).fold(std::u32::MIN, |a,b| a.max(b)) as usize;


    //common::stream_or_stdout(None);


    let num_nodes = nodes_reader.read_u32::<LittleEndian>().unwrap() as usize;
    let num_edges = nodes_reader.read_u32::<LittleEndian>().unwrap();
    info!("num_nodes : {}", num_nodes);
    info!("num_edges : {}", num_edges);


    let mut adjacency_lists = vec![Vec::with_capacity(1); num_nodes];
    let mut current_node = 0;

    let mut edge_counter : u64 = 0;
    let mut max_adjacency_to_node : u32 = 0;
    loop {
        match nodes_reader.read_i32::<LittleEndian>() {
            Err(_) => break,
            Ok(field) => {
                if field < 0 { 
                    current_node = -field-1;
                    assert_lt!(current_node as usize, adjacency_lists.len());
                    continue;
                }
                assert_gt!(field, 0);
                let element = field as u32 -1;
                if element > max_adjacency_to_node { max_adjacency_to_node = element; }
                adjacency_lists[current_node as usize].push(element);
                edge_counter += 1;
            }
        }
    }
    assert_eq!(edge_counter, num_edges as u64);

    info!("length of adjacency list: {}", adjacency_lists.len());
    info!("maximum biclique node id: {}", max_biclique_from_node);

    let node_degrees = {
        let from_node_bound = std::cmp::max(num_nodes,max_biclique_from_node+1);
        let mut node_degrees = vec![0; from_node_bound];
        for row_id in 0..adjacency_lists.len() {
            node_degrees[row_id] += adjacency_lists[row_id].len();
        }
        for biclique_id in 0..from_bicliques.len() {
            for node in &from_bicliques[biclique_id] {
                node_degrees[*node as usize] += to_bicliques[biclique_id].len();
            }
        }
        node_degrees
    };

    println!("RESULT file={file} action=load algo=rustpagerank bicliques={bicliques} time_ms={time} ", 
             time=now.elapsed().as_millis(),
             file=input_filename,
             bicliques=clique_filename
             );



    match matches.value_of("output") {
        None =>  {
            let to_node_bound = std::cmp::max(adjacency_lists.len(), std::cmp::max(max_adjacency_to_node as usize, max_biclique_to_node) + 1); 
            let mut input_vector = vec![1.; to_node_bound];
            for iter in 0..10 {
                let now = Instant::now();
                let mut output_vector = vec![0.15; to_node_bound];
                for row_id in 0..adjacency_lists.len() {
                    for node in &adjacency_lists[row_id] {
                        output_vector[row_id] += input_vector[*node as usize];
                    }
                }
                for biclique_id in 0..from_bicliques.len() {
                    let result = {
                        let mut result = 0.;
                        for to_node in &to_bicliques[biclique_id] {
                            result += input_vector[*to_node as usize];
                        }
                        result
                    };
                    for from_node in &from_bicliques[biclique_id] {
                        output_vector[*from_node as usize] += result;
                    }
                }
                let mut hash = 0.;
                for i in 0..output_vector.len() {
                    if i >= node_degrees.len() || node_degrees[i] == 0 { continue; }
                    output_vector[i] = 0.85*output_vector[i]/(node_degrees[i] as f64);
                    hash += output_vector[i];
                }
                std::mem::swap(&mut input_vector, &mut output_vector);
                println!("RESULT file={file} action=pagerank algo=rustpagerank bicliques={bicliques} time_ns={time} hash={hash} iter={iter}", 
                         file=input_filename,
                         time=now.elapsed().as_nanos(),
                         bicliques=clique_filename,
                         hash=hash,
                         iter=iter
                         );
            }
        },
        Some(_) => {
            let mut node2to_biclique = std::collections::HashMap::new(); //@ stores for each node ID on the left side of a biclique the index in the biclique array storing all nodes on the right side

            // let mut bicliques = Vec::<Vec<u32>>::new();
            for i in 0..from_bicliques.len() {
                for node in &from_bicliques[i] {
                    node2to_biclique.insert(node, i); 
                }
            }
            let mut writer = common::stream_or_stdout(matches.value_of("output"));
            writeln!(writer, "{}", std::cmp::max(num_nodes,max_biclique_from_node+1)).unwrap();
            for row_id in 0..adjacency_lists.len() {
                let row = {
                    if node2to_biclique.contains_key(&(row_id as u32)) {
                        let mut row = adjacency_lists[row_id].clone();
                        row.append(&mut (to_bicliques[node2to_biclique[&(row_id as u32)] as usize].clone()));
                        row.retain(|&x| x != row_id as u32);
                        row.sort();
                        row
                    } else {
                        adjacency_lists[row_id].clone()
                    }
                };
                writer.write(row.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ").as_bytes()).unwrap();
                if row.len() > 0 {
                    writer.write(" ".as_bytes()).unwrap();
                }
                writer.write("\n".as_bytes()).unwrap();
            }
            for row_id in adjacency_lists.len()..max_biclique_from_node+1 {
                let mut row = to_bicliques[node2to_biclique[&(row_id as u32)] as usize].clone();
                row.retain(|&x| x != row_id as u32);
                writer.write(row.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ").as_bytes()).unwrap();
                if row.len() > 0 {
                    writer.write(" ".as_bytes()).unwrap();
                }
                writer.write("\n".as_bytes()).unwrap();
            }
            writer.flush().unwrap();
        }
    }

}
