extern crate smfr;

fn main() {
	use std::env;
	let args: Vec<String> = env::args().collect();

	match args.get(1) {
		None => {
			eprintln!("Command not specified!");
			std::process::exit(1);
		},

		Some(cmd) => {
			if cmd == "expand" {

				let readpath = args.get(2);
				let writepath = args.get(3);
				if readpath.is_none() || writepath.is_none() {
					eprintln!("Not enough arguments");
					std::process::exit(1);
				}

				match expand(readpath.unwrap(), writepath.unwrap()) {
					Ok(_) => std::process::exit(0),
					Err(e) => {
						eprintln!("Error: {}", e);
						std::process::exit(0);
					}
				}
			}
		}
	}
}

fn expand(read: &str, write: &str) -> Result<(), String> {
	use smfr::file::filerw;
	use smfr::file::parser;
	use std::path::Path;

	let reader = filerw::SmfReader::read_from_file(Path::new(read));
    match reader {
        Ok(r) => {
            let mut parser = parser::SmfParser::new(r);
            let smf = parser.read_all().unwrap();
            println!("Parsed midi file");
            println!("Writing to {}", write);
            filerw::write_to_file(Path::new(write), &smf, true).expect("Writing failed");
            Ok(())
        },

        Err(e) => {
            Err(e.to_string())
        }
    }
}