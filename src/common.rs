

/// open an input file or use stdin if no filename is given 
pub fn stream_or_stdin(filename : Option<&str>) -> Box<dyn std::io::Read> { 
	match filename {
		Some(filename) => { 
			// info!("filename: {}", filename); 
			let path = std::path::Path::new(filename);
			Box::new(std::io::BufReader::new(std::fs::File::open(&path).unwrap())) as Box<dyn std::io::Read>
		} 
		None => Box::new(std::io::stdin()) as Box<dyn std::io::Read>, 
	} 
} 


/// open an file for output or use stdout if no filename is given 
pub fn stream_or_stdout(filename : Option<&str>) -> Box<dyn std::io::Write> { 
	match filename {
		Some(filename) => { 
			// info!("filename: {}", filename); 
			let path = std::path::Path::new(filename);
			Box::new(std::fs::File::create(&path).unwrap()) as Box<dyn std::io::Write>
		} 
		None => Box::new(std::io::stdout()) as Box<dyn std::io::Write>
	} 
} 
