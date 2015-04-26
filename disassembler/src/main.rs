#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter};
use std::io::prelude::*;
use std::path::Path;
use std::fmt;

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
  match args {
    Args { flag_version: true, .. } => println!("version {}", VERSION),
    Args {flag_input: input_file_path, flag_output: output_file_path, ..} => decode_file(input_file_path, output_file_path),
  }
}

fn decode_file(input_file_path: String, ouput_file_path: String) {
  let input_path = Path::new(&input_file_path);
  let output_path = Path::new(&ouput_file_path);
  let display = input_path.display();

  let mut input_file = match File::open(&input_path) {
    Err(why) => panic!("could not open {}: {}", display, Error::description(&why)),
    Ok(file) => file,
  };

  let mut buffer: Vec<u8> = Vec::new();
  let file_size_res = input_file.read_to_end(& mut buffer);

  let mut openOptions = OpenOptions::new();
  openOptions.write(true).append(true);

  let mut ouput_file = match File::create(output_path) {
    Err(why) => panic!("could not create {}", why),
    Ok(file) => file,
  };

  if let Ok(file_size) = file_size_res {  
    let mut program_counter = 0;
    while((program_counter as usize) < file_size) {
      program_counter += disassemble(&buffer, program_counter, &mut ouput_file);
    }
  }

}

fn disassemble(instruction_buffer: & Vec<u8>, program_counter: i32, output_file: &mut File) -> i32 {

  let operation_code = instruction_buffer[program_counter as usize];
  let operation_arg1 = instruction_buffer[(program_counter + 1) as usize];
  let operation_arg2 = instruction_buffer[(program_counter + 2) as usize];
  let mut operation_size = 1;

  let mut output = Vec::new();
  write!(&mut output, "{:01$x} \t", program_counter, 4);

  match operation_code {
    0x00 => { write!(&mut output,"NOP\n"); },
    0xc3 => { write!(&mut output,"JMP \t${:x}{:x}\n", operation_arg2, operation_arg1); operation_size = 3 },
    0xf5 => { write!(&mut output,"PUSH \tPSW\n"); },
    _ => panic!("can not decode: {:x}", operation_code)
  }

  output_file.write_all(&mut output);
  return operation_size;
}
