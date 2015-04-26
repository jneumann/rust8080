#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

docopt!(Args derive Debug, "
8080 Disassembler – let's you disassemble a 8080 binary

Usage:
  disassembler -i IFILE -o OFILE
  disassembler -h | --help
  disassembler -v | --version

Options:
  -i IFILE --input=IFILE     Specify the input file
  -o OFILE --output=OFILE    Specivy the output file
  -h --help                  Show this screen.
  -v --version               Show version.
");

static VERSION: &'static str = "0.0.1";

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
//    println!("{:?}", args);
    match args {
        Args { flag_version: true, .. } => println!("version {}", VERSION),
        Args {flag_input: input_file_path, flag_output: output_file_path, ..} => decode_file(input_file_path, output_file_path),
    }
}

fn decode_file(input_file_path: String, ouput_file_path: String) {
	let input_path = Path::new(&input_file_path);
	let display = input_path.display();

	let mut file = match File::open(&input_path) {
		Err(why) => panic!("could not open {}: {}", display, Error::description(&why)),
		Ok(file) => file,
	};

	let mut s = String::new();
	match file.read_to_string(&mut s) {
		Err(why) => panic!("could not read {}: {}", display, Error::description(&why)),
		Ok(_) => print!("{} contains:\n{}", display, s),
	}
}
