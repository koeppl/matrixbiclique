extern crate env_logger;

#[macro_use] extern crate clap;
extern crate log;
use log::info;

use std::io::prelude::*;
use std::io::BufRead;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

#[macro_use] extern crate more_asserts;


#[allow(dead_code)] mod common;


fn main() {
    let matches = clap_app!(count_r =>
        (about: "computes Cecilia's bin-format from an ASCIIGraph")
        (@arg input:  -i --infile  +takes_value "the input file to read (otherwise read from stdin)")
        (@arg output: -o --outfile  +takes_value +required "filename for the binary output")
    ).get_matches();

    let mut edge_counter : u64 = 0;
    let output_filename = &matches.value_of("output").unwrap();

    {
        let reader = std::io::BufReader::new(common::stream_or_stdin(matches.value_of("input")));
        let mut writer = std::io::BufWriter::new(
            std::fs::OpenOptions::new().write(true).truncate(true).read(false).create(true).open(output_filename).expect("no file found")
        );
        //common::stream_or_stdout(None);



        info!("read");
        let mut lines_it = reader.lines();
        let number_of_nodes = lines_it.next().unwrap().unwrap().parse::<u32>().unwrap();
        writer.write_u32::<LittleEndian>(number_of_nodes).unwrap();
        writer.write_u32::<LittleEndian>(0).unwrap(); // number of edges, will be filled afterwards


        let mut line_no : u64 = 1;
        for line in lines_it {
            writer.write_i32::<LittleEndian>(-(line_no as i32)).unwrap(); 
            for number_string in line.unwrap().split(' ') {
                let y = number_string.trim();
                if y.len() == 0 { continue; }

                let number = y.parse::<u32>().unwrap()+1;
                assert_lt!(number, std::u32::MAX);
                writer.write_u32::<LittleEndian>(number).unwrap(); 
                edge_counter += 1;
            }
            assert_lt!(line_no, std::u32::MAX as u64);
            assert_lt!(edge_counter, std::u32::MAX as u64);
            line_no += 1;
        }
        writer.flush().unwrap();
    }
    let mut write_handle = std::fs::OpenOptions::new().write(true).truncate(false).read(true).create(false).open(output_filename).expect("no file found");
    assert_eq!( {
    write_handle.seek(std::io::SeekFrom::Start(4)).unwrap();
    write_handle.read_u32::<LittleEndian>().unwrap()
    }, 0u32);

    write_handle.seek(std::io::SeekFrom::Start(4)).unwrap();
    write_handle.write_u32::<LittleEndian>(edge_counter as u32).unwrap();

    // let bwt = match use_matrix { 
    //     true => compute_bwt_matrix(&text),
    //     false => compute_bwt(&text) 
    // };
    // let r = number_of_runs(&bwt);
    // println!("RESULT algo=bwt time_ms={} length={} bwt_runs={} file={} no_dollar={} use_matrix={}", now.elapsed().as_millis(), bwt.len(), r, matches.value_of("input").unwrap_or("stdin"), no_dollar, use_matrix);

}
