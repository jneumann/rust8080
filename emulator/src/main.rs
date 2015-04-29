#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

use std::io::prelude::*;
use std::fs::{File};

// docopt!(Args derive Debug, "
//   8080 Emulator – let's you emulat an intel 8080 CPU

//   Usage:
//   disassembler -i IFILE -o OFILE
//   disassembler -h | --help
//   disassembler -v | --version

//   Options:
//   -i IFILE --input=IFILE     Specify the input file
//   -o OFILE --output=OFILE    Specivy the output file
//   -h --help                  Show this screen.
//   -v --version               Show version.
//   ");

// static VERSION: &'static str = "0.0.1";

struct ConditionCode {
    /// Zero: set if the result is zero
    z: u8,
    /// Sign: set if the result is negative
    s: u8,
    /// Parity: set if the number of 1 bits in the result is even
    p: u8,
    /// Carry: set if the last addition operation resulted in a carry, or
    /// if the last subtraction operation required a borrow
    cy: u8,
    ac: u8,
    pad: u8,
}

struct CpuState {
    // Register A: primary 8-bit accumulator
    a: u8,
    // Register B: either 8-bit single or B (BC) 16-bit register
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    memory: [u8; 0x10000],   //store 'memory' on heap?
    cc: ConditionCode,
    int_enable: u8,
}


fn main() {
  // let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
  // match args {
  //   Args { flag_version: true, .. } => println!("version {}", VERSION),
  //   Args {flag_input: input_file_path, flag_output: output_file_path, ..} => decode_file(input_file_path, output_file_path),
  // }

  println!("running emulator");
  let mut cpu_state = init_cpu();
  load_rom_to_memory(&mut cpu_state);

  let mut done: i32 = 0;

  while done == 0 {
    // println!("emulate");
    done = emulate(&mut cpu_state);
  }
}



fn load_rom_to_memory(cpu_state: &mut CpuState) {

  // let mut input_file = File::open("invaders.h").unwrap();
  let mut input_file = File::open("invaders.rom").unwrap();

  let mut buffer: Vec<u8> = Vec::new();
  let file_size = input_file.read_to_end(&mut buffer).unwrap();

  for (idx, byte) in buffer.iter().enumerate() {
    cpu_state.memory[idx] = byte.clone();
  }
}

fn init_cpu() -> CpuState {

  let con_code = ConditionCode{ z:0x00, s:0x00, p:0x00, cy:0x00, ac: 0x00, pad: 0x00, };

  let cpu_state = CpuState{ 
    a:0x00,
    b:0x00,
    c:0x00,
    d:0x00,
    e:0x00,
    h:0x00,
    l:0x00,
    sp:0x0000,
    pc:0x0000,
    memory: [0; 0x10000],
    cc: con_code,
    int_enable: 0,
  };

  cpu_state
}

fn emulate(cpu_state: &mut CpuState) -> i32 {

  // println!("run emulator");

  if cpu_state.pc == 0x2000 {
    println!("no more code to execute");
    return 1;
  }

  // println!("code left");
  disassemble(&cpu_state.memory, cpu_state.pc);

  let operation_code = cpu_state.memory[cpu_state.pc as usize];
  // possible out of bounds?
  let operation_arg1 = cpu_state.memory[(cpu_state.pc + 1) as usize];
  let operation_arg2 = cpu_state.memory[(cpu_state.pc + 2) as usize];

  let mut operation_cycles = 0;

  // println!("oa1: {:01$x}", operation_arg1, 2);
  // println!("oa2: {:01$x}", operation_arg2, 2);

  // println!("ca: {:01$x}", (operation_arg2 as u16) << 8 | (operation_arg1 as u16), 4);

  cpu_state.pc+=1;

  match operation_code {

    //NOP ;4c ;os=1byte
    0x00 => { operation_cycles = 4 },

    //DRC B ; 5c; os=1byte decrement register B
    // 0x05 => { }

    // MVI B, u8  Move immediate value to B ;os=2byte
    0x06 => { cpu_state.b = operation_arg1; cpu_state.pc+=1; }

    //LXI D, u16 ; 10c; os=3byte  load intermediate to combined register DE (just called D as 16 bit register)  
    0x11 => { cpu_state.e = operation_arg1; cpu_state.d = operation_arg2; cpu_state.pc += 2; operation_cycles = 10; },

    //INX D ;5c; os=1byte increment register DE
    0x13 => { 
      cpu_state.e += 1;
      if cpu_state.e == 0 {
        cpu_state.h += 1;
      }

      operation_cycles = 5;
    },

    //LDAX D load ; 7c; os=1byte; load memory indirect from combinded register DE (just called D as 16 bit register) to A 
    0x1a => { 
      let memory_offset: u16 = ((cpu_state.d as u16) << 8) | (cpu_state.e as u16); 
      // println!("{:01$x}", memory_offset, 4);
      // println!("{:01$x}", cpu_state.memory[memory_offset as usize], 2);
      cpu_state.a = cpu_state.memory[memory_offset as usize];
      operation_cycles = 7; 
    },

    //LXI H, u16 ; 10c; os=3byte  load intermediate to combined register HL (just called H as 16 bit register)
    0x21 => { cpu_state.l = operation_arg1; cpu_state.h = operation_arg2; cpu_state.pc += 2; operation_cycles = 10; },

    //INX H ;5c; os=1byte  increment register HL
    0x23 => { 
      cpu_state.l += 1; 
      if cpu_state.l == 0 {
        cpu_state.h += 1;
      }

      operation_cycles = 5;
    },

    //LXI sp, u16   Load registerpair u16 immediate to stack pointer(which is u16) ;10c ;os=3byte
    0x31 => { cpu_state.sp = (operation_arg2 as u16) << 8 | (operation_arg1 as u16); cpu_state.pc += 2; operation_cycles = 10; },

    //MOV M,A ; 7c; os=1  move content of register A to memory location pointed by register HL 16 bit ...
    0x77 => {  
      let memory_offset: u16 = ((cpu_state.h as u16) << 8) | (cpu_state.l as u16);
      cpu_state.memory[memory_offset as usize] = cpu_state.a;
      // println!("{:01$x}", memory_offset, 4);
      // println!("{:01$x}", cpu_state.memory[memory_offset as usize], 2);
      operation_cycles = 7; 
    },

    //JMP u16  jump to u16 adress ;10c ; os=3byte
    0xc3 => { cpu_state.pc = (operation_arg2 as u16) << 8 | (operation_arg1 as u16); operation_cycles = 10; },

    //CALL adr u16 ;17; os=3byte
    0xcd => {  
      let ret: u16 = cpu_state.pc + 2; // save return adress (3 byte after this 3 byte instr.) on the stack
      // println!("{:01$x}", (ret >> 8) as u8, 4);
      // println!("{:01$x}", ret as u8, 4);
      cpu_state.memory[(cpu_state.sp - 1) as usize] = (ret >> 8) as u8; // -- as u8 == & 0xff -- bitmask lower 8 bits of return addr. to write in higher stack bits
      cpu_state.memory[(cpu_state.sp - 2) as usize] = ret as u8; // -- as u8 == & -- 0xff bitmask higher 8 bits to write to lower stack bits
      // println!("{:01$x}", cpu_state.memory[(cpu_state.sp - 1) as usize], 2);
      // println!("{:01$x}", cpu_state.memory[(cpu_state.sp - 2) as usize], 2);

      cpu_state.sp -= 2;  // stack grows down
      cpu_state.pc = (operation_arg2 as u16) << 8 | (operation_arg1 as u16); // jump to destination
      
      operation_cycles = 17;
    },
    
    _ => panic!("the opcode: {:01$x} is not yet implemented, shutting down vm.", operation_code, 2),
  }
  println!("z:{:?} s:{:?} p:{:?} cy:{:?} ac:{:?}",cpu_state.cc.z, cpu_state.cc.s, cpu_state.cc.p, cpu_state.cc.cy, cpu_state.cc.ac );
  println!("A:{:09$x} B:{:09$x} C:{:09$x} D:{:09$x} E:{:09$x} H:{:09$x} L:{:09$x} SP:{:010$x} PC:{:010$x}", cpu_state.a, cpu_state.b, cpu_state.c, cpu_state.d, cpu_state.e, cpu_state.h, cpu_state.l, cpu_state.sp, cpu_state.pc, 2, 4);
  println!("");
  return 0;
}


fn disassemble(instruction_buffer: &[u8], program_counter: u16) -> u16 {

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
    0x04 => { write!(&mut output, "INR \tB\n");},
    0x05 => { write!(&mut output, "DCR \tB\n");},
    0x06 => { write!(&mut output, "MVI \tB, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x07 => { write!(&mut output, "RLC\n"); },
    0x0d => { write!(&mut output, "DCR \tC\n");},
    0x0e => { write!(&mut output, "MVI \tC, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x0f => { write!(&mut output, "RRC\n"); },

    0x11 => { write!(&mut output, "LXI \tD, #${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x13 => { write!(&mut output, "INX \tD\n");},
    0x14 => { write!(&mut output, "INR \tD\n");},
    0x15 => { write!(&mut output, "DCR \tD\n");},
    0x16 => { write!(&mut output, "MVI \tD, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x19 => { write!(&mut output, "DAD \tD\n");},
    0x1a => { write!(&mut output, "LDAX \tD\n");},

    0x20 => { write!(&mut output, "NOP\n"); },
    0x21 => { write!(&mut output, "LXI \tH, #${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x22 => { write!(&mut output, "SHLD \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x23 => { write!(&mut output, "INX \tH\n");},
    0x27 => { write!(&mut output, "DAA \n"); },
    0x2a => { write!(&mut output, "LHLD \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x2b => { write!(&mut output, "DCX \tH\n");},
    0x2c => { write!(&mut output, "INR \tL\n");},
    0x2e => { write!(&mut output, "MVI \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },

    0x31 => { write!(&mut output, "LXI \tSP, ${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x32 => { write!(&mut output, "STA \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x34 => { write!(&mut output, "INR \tM\n");},
    0x35 => { write!(&mut output, "DCR \tM\n");},
    0x36 => { write!(&mut output, "MVI \tM, #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x3a => { write!(&mut output, "LDA \t${:02$x}{:02$x}\n", operation_arg2, operation_arg1, 2); operation_size = 3 },
    0x3c => { write!(&mut output, "INR \tA\n");}
    0x3d => { write!(&mut output, "DCR \tA\n");}
    0x3e => { write!(&mut output, "MVI \tA, #${:01$x}\n", operation_arg1, 2); operation_size = 2},

    0x46 => { write!(&mut output, "MOV \tB, M\n");},
    0x47 => { write!(&mut output, "MOV \tB, A\n");},
    0x4e => { write!(&mut output, "MOV \tC, M\n");},
    0x4f => { write!(&mut output, "MOV \tC, A\n");},

    0x56 => { write!(&mut output, "MOV \tD, M\n");},
    0x5e => { write!(&mut output, "MOV \tE, M\n");},
    0x5f => { write!(&mut output, "MOV \tE, A\n");},

    0x61 => { write!(&mut output, "MOV \tH, C\n");},
    0x66 => { write!(&mut output, "MOV \tH, M\n");},
    0x67 => { write!(&mut output, "MOV \tH, A\n");},
    0x68 => { write!(&mut output, "MOV \tL, B\n");},
    0x6f => { write!(&mut output, "MOV \tL, A\n");},

    0x70 => { write!(&mut output, "MOV \tM, B\n");},
    0x72 => { write!(&mut output, "MOV \tM, D\n");},
    0x73 => { write!(&mut output, "MOV \tM, E\n");},
    0x77 => { write!(&mut output, "MOV \tM, A\n");},
    0x78 => { write!(&mut output, "MOV \tA, B\n");},
    0x79 => { write!(&mut output, "MOV \tA, C\n");},
    0x7a => { write!(&mut output, "MOV \tA, D\n");},
    0x7b => { write!(&mut output, "MOV \tA, E\n");},
    0x7d => { write!(&mut output, "MOV \tA, L\n");},
    0x7e => { write!(&mut output, "MOV \tA, M\n");},

    0x85 => { write!(&mut output, "ADD \tL\n");},
    0x86 => { write!(&mut output, "ADD \tM\n");},
    0x8d => { write!(&mut output, "ADC \tL\n");},

    0xa1 => { write!(&mut output, "ANA \tC\n");},
    0xa7 => { write!(&mut output, "ANA \tA\n");},
    0xaf => { write!(&mut output, "XRA \tA\n");},

    0xb0 => { write!(&mut output, "DCX \tB\n");},

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
    _ => panic!("can not disassemble: {:01$x}", operation_code, 2)
  }

  let s = String::from_utf8(output).unwrap();
  print!("{}", s);
  return operation_size;
}