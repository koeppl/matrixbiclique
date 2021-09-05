extern crate env_logger;

extern crate log;
use log::{info,debug};

use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

#[macro_use] extern crate more_asserts;


#[allow(dead_code)] mod common;


fn main() {
    // let matches = clap_app!(count_r =>
    //     (about: "computes the BWT via divsufsort")
    //     (@arg input:  -i --infile  +takes_value "the input file to read (otherwise read from stdin")
    //     (@arg input:  -o --outfile  +takes_value "the binary output file (otherwise writo to stdout")
    // ).get_matches();
    //
    // let reader = stream_or_stdin(&matches.value_of("input"));
    // let writer = stream_or_stdout(&matches.value_of("output"));

    let mut edge_counter : u64 = 0;
    let mut vertex_counter: u64 = 0;

    {
        let reader = std::io::BufReader::new(common::stream_or_stdin(None));
        let mut writer = std::io::BufWriter::new(
            std::fs::OpenOptions::new().write(true).read(false).create(true).open("output.bin").expect("no file found")
        );
        //common::stream_or_stdout(None);



        info!("read");

        writer.write_u32::<LittleEndian>(0).unwrap(); // number of nodes, will be filled afterwards
        writer.write_u32::<LittleEndian>(0).unwrap(); // number of edges, will be filled afterwards


        for line in reader.lines() {
            let parsed_line = line.unwrap();

            let splittedline = parsed_line.split(':').collect::<Vec<&str>>();
            assert_eq!(splittedline.len(), 2);
            let line_no = splittedline[0].trim().parse::<u64>().unwrap();
            assert_gt!(line_no, 0);
            assert_lt!(line_no, std::u32::MAX as u64);
            writer.write_i32::<LittleEndian>(-(line_no as i32)).unwrap(); 
            if line_no > vertex_counter { vertex_counter = line_no; }
            println!("lineno: {}", line_no);

            for number in splittedline[1].split(' ').map(|x| -> Option<u32> { 
                let y = x.trim();
                if y.len() > 0  {
                    Some(y.parse::<u32>().unwrap())
                } else {
                    None
                }
            }) {
                match number {
                    None => (),
                    Some(num) => {
                        assert_lt!(num, std::u32::MAX);
                        if num > 0 && num != line_no as u32 {
                            writer.write_u32::<LittleEndian>(num).unwrap(); 
                            edge_counter += 1;
                            println!("write {}", num);
                        }
                    }
                }
            }
            assert_lt!(edge_counter, std::u32::MAX as u64);
        }
        writer.flush().unwrap();
    }
    let mut write_handle = std::fs::OpenOptions::new().write(true).read(true).create(false).open("output.bin").expect("no file found");
    assert_eq!( {
    write_handle.read_u64::<LittleEndian>().unwrap()
    }, 0u64);

    write_handle.seek(std::io::SeekFrom::Start(0)).unwrap();
    write_handle.write_u32::<LittleEndian>(vertex_counter as u32).unwrap();
    write_handle.write_u32::<LittleEndian>(edge_counter as u32).unwrap();

    // let bwt = match use_matrix { 
    //     true => compute_bwt_matrix(&text),
    //     false => compute_bwt(&text) 
    // };
    // let r = number_of_runs(&bwt);
    // println!("RESULT algo=bwt time_ms={} length={} bwt_runs={} file={} no_dollar={} use_matrix={}", now.elapsed().as_millis(), bwt.len(), r, matches.value_of("input").unwrap_or("stdin"), no_dollar, use_matrix);

}
