#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

use std::error::Error;
use std::fs::{File, OpenOptions};
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
  match args {
    Args { flag_version: true, .. } => println!("version {}", VERSION),
    Args { flag_input: input_file_path, flag_output: output_file_path, .. } => decode_file(input_file_path, output_file_path),
  }
}

fn decode_file(input_file_path: String, output_file_path: String) {
  let input_path = Path::new(&input_file_path);
  let output_path = Path::new(&output_file_path);
  let display = input_path.display();

  let mut input_file = match File::open(&input_path) {
    Err(why) => panic!("could not open {}: {}", display, Error::description(&why)),
    Ok(file) => file,
  };

  let mut buffer: Vec<u8> = Vec::new();
  let file_size_res = input_file.read_to_end(& mut buffer);

  let mut open_options = OpenOptions::new();
  open_options.write(true).append(true);

  let mut ouput_file = match File::create(output_path) {
    Err(why) => panic!("could not create {}", why),
    Ok(file) => file,
  };

  if let Ok(file_size) = file_size_res {  
    let mut program_counter = 0;
    while (program_counter as usize) < file_size {
      program_counter += disassemble(&buffer, program_counter, &mut ouput_file);
    }
  }

}

fn disassemble(instruction_buffer: & Vec<u8>, program_counter: i32, output_file: &mut File) -> i32 {

  let operation_code = instruction_buffer[program_counter as usize];
  // possible out of bounds?
  let operation_arg1 = instruction_buffer[(program_counter + 1) as usize];
  let operation_arg2 = instruction_buffer[(program_counter + 2) as usize];
  let mut operation_size = 1;

  let mut output = Vec::new();
  write!(&mut output, "{:01$x}: \t", program_counter, 4);

  match operation_code {
    0x00 => { write!(&mut output, "NOP\n"); },
    0x01 => { write!(&mut output, "LXI \tB, #${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x04 => { write!(&mut output, "INR \tB\n");}
    0x05 => { write!(&mut output, "DCR \tB\n");}
    0x06 => { write!(&mut output, "MVI \tB, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x07 => { write!(&mut output, "RLC\n"); },
    0x0d => { write!(&mut output, "DCR \tC\n");}
    0x0e => { write!(&mut output, "MVI \tC, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x0f => { write!(&mut output, "RRC\n"); },

    0x11 => { write!(&mut output, "LXI \tD, #${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x14 => { write!(&mut output, "INR \tD\n");}
    0x15 => { write!(&mut output, "DCR \tD\n");}
    0x16 => { write!(&mut output, "MVI \tD, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x19 => { write!(&mut output, "DAD \tD\n");}

    0x20 => { write!(&mut output, "NOP\n"); },
    0x21 => { write!(&mut output, "LXI \tH, #${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x22 => { write!(&mut output, "SHLD \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x23 => { write!(&mut output, "INX \tH\n");}
    0x27 => { write!(&mut output, "DAA \n"); },
    0x2a => { write!(&mut output, "LHLD \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x2b => { write!(&mut output, "DCX \tH\n");}
    0x2c => { write!(&mut output, "INR \tL\n");},
    0x2e => { write!(&mut output, "MVI \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },

    0x31 => { write!(&mut output, "LXI \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x32 => { write!(&mut output, "STA \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x34 => { write!(&mut output, "INR \tM\n");},
    0x35 => { write!(&mut output, "DCR \tM\n");},
    0x36 => { write!(&mut output, "MVI \tM, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x3a => { write!(&mut output, "LDA \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x3c => { write!(&mut output, "INR \tA\n");}
    0x3d => { write!(&mut output, "DCR \tA\n");}
    0x3e => { write!(&mut output, "MVI \tA, #${:01$x}\n", operation_arg1, 2); operation_size = 2},

    0x46 => { write!(&mut output, "MOV \tB, M\n");}
    0x47 => { write!(&mut output, "MOV \tB, A\n");}
    0x4e => { write!(&mut output, "MOV \tC, M\n");}
    0x4f => { write!(&mut output, "MOV \tC, A\n");}

    0x56 => { write!(&mut output, "MOV \tD, M\n");}
    0x5e => { write!(&mut output, "MOV \tE, M\n");}
    0x5f => { write!(&mut output, "MOV \tE, A\n");}

    0x61 => { write!(&mut output, "MOV \tH, C\n");}
    0x66 => { write!(&mut output, "MOV \tH, M\n");}
    0x67 => { write!(&mut output, "MOV \tH, A\n");}
    0x68 => { write!(&mut output, "MOV \tL, B\n");}
    0x6f => { write!(&mut output, "MOV \tL, A\n");}

    0x70 => { write!(&mut output, "MOV \tM, B\n");}
    0x72 => { write!(&mut output, "MOV \tM, D\n");}
    0x73 => { write!(&mut output, "MOV \tM, E\n");}
    0x77 => { write!(&mut output, "MOV \tM, A\n");}
    0x78 => { write!(&mut output, "MOV \tA, B\n");}
    0x79 => { write!(&mut output, "MOV \tA, C\n");}
    0x7a => { write!(&mut output, "MOV \tA, D\n");}
    0x7b => { write!(&mut output, "MOV \tA, E\n");}
    0x7d => { write!(&mut output, "MOV \tA, L\n");}
    0x7e => { write!(&mut output, "MOV \tA, M\n");}

    0x85 => { write!(&mut output, "ADD \tL\n");}
    0x86 => { write!(&mut output, "ADD \tM\n");}
    0x8d => { write!(&mut output, "ADC \tL\n");}

    0xa7 => { write!(&mut output, "ANA \tA\n");}
    0xaf => { write!(&mut output, "XRA \tA\n");}

    0xb0 => { write!(&mut output, "DCX \tB\n");}

    0xc0 => { write!(&mut output, "RNZ \n"); },
    0xc1 => { write!(&mut output, "POP \tB \n"); },
    0xc2 => { write!(&mut output, "JNZ \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xc3 => { write!(&mut output, "JMP \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xc4 => { write!(&mut output, "CNZ \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xc5 => { write!(&mut output, "PUSH \tB\n"); },
    0xc6 => { write!(&mut output, "ADI \t#${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0xc8 => { write!(&mut output, "RZ \n"); },
    0xc9 => { write!(&mut output, "RET \n"); },
    0xca => { write!(&mut output, "JZ \t\t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xcc => { write!(&mut output, "CZ \t\t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xcd => { write!(&mut output, "CALL \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },

    0xd0 => { write!(&mut output, "RNC \tD\n"); },
    0xd1 => { write!(&mut output, "POP \tD\n"); },
    0xd2 => { write!(&mut output, "JNC \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xd3 => { write!(&mut output, "OUT \t#${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0xd5 => { write!(&mut output, "PUSH \tD\n"); },
    0xda => { write!(&mut output, "JC \t\t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xdb => { write!(&mut output, "IN \t\t#${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0xde => { write!(&mut output, "SBI \t#${:01$x}\n", operation_arg1, 2); operation_size = 2},

    0xe1 => { write!(&mut output, "POP \tH \n"); },
    0xe3 => { write!(&mut output, "XTHL \n"); },
    0xe5 => { write!(&mut output, "PUSH \tH\n"); },
    0xe6 => { write!(&mut output, "ANI \t#${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0xe9 => { write!(&mut output, "PCHL \n"); },
    0xeb => { write!(&mut output, "XCHG \n"); },

    0xf1 => { write!(&mut output, "POP \tPSW \n"); },
    0xf5 => { write!(&mut output, "PUSH \tPSW\n"); },
    0xfa => { write!(&mut output, "JM \t\t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0xfb => { write!(&mut output, "EI \n"); },
    0xfe => { write!(&mut output, "CPI \t#${:01$x}\n", operation_arg1, 2); operation_size = 2},
    _ => panic!("can not decode: {:01$x}", operation_code, 2)
  }

  output_file.write_all(&mut output);
  return operation_size;
}
