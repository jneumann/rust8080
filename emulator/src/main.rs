#![feature(plugin)]
#![plugin(docopt_macros)]

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

extern crate rustc_serialize;
extern crate docopt;

use std::io::prelude::*;
use std::fs::{File};
use std::num::Wrapping;

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
    z: bool,
    /// Sign: set if the result is negative
    s: bool,
    /// Parity: set if the number of 1 bits in the result is even
    p: bool,
    /// Carry: set if the last addition operation resulted in a carry, or
    /// if the last subtraction operation required a borrow
    cy: bool,
    ac: bool,
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
  let mut debug_instruction_ctx: i32 = 0;

  while done == 0 {
    // println!("emulate");
    done = emulate(&mut cpu_state);
    debug_instruction_ctx += 1;
    // println!("instr_ctx: {:?} \n", debug_instruction_ctx);

    //breakpoint
    if debug_instruction_ctx == 1548 {
      // panic!("breakpoint");
    }
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

  let con_code = ConditionCode{ z:false, s:false, p:false, cy:false, ac:false, };

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
    0x00 => { operation_cycles = 4; },

    //LXI B, u16   load immediate register pair BC   10c; os=3
    0x01 => {  
      cpu_state.c = operation_arg1;
      cpu_state.b = operation_arg2;
      cpu_state.pc += 2;

      operation_cycles = 10; 
    },

    //DRC B ; 5c; os=1byte decrement register B
    0x05 => { 
      let res = (Wrapping(cpu_state.b) - Wrapping(1)).0; // allow integer overflow
      cpu_state.cc.z = res == 0;
      cpu_state.cc.s = 0x80 == (res & 0x80);
      cpu_state.cc.p = parity(res, 8);
      cpu_state.b = res;
    }

    //MVI B, u8  Move immediate value to B ;os=2byte
    0x06 => { cpu_state.b = operation_arg1; cpu_state.pc += 1; }

    //DAD B  add register pair BC to HL  ; 10c, ox=1
    0x09 => {  

      cpu_state.b = 0x01;

      let bc: u32 = (cpu_state.b as u32) << 8 | cpu_state.c as u32;
      let hl: u32 = (cpu_state.h as u32) << 8 | cpu_state.l as u32;

      // println!("bc: {:01$x}", bc, 4);
      // println!("hl: {:01$x}", hl, 4);

      let res: u32 = bc + hl;

      // println!("res: {:01$x}", res, 4);

      cpu_state.h = ((res & 0xff00) >> 8) as u8;
      cpu_state.l = (res & 0xff) as u8;
      cpu_state.cc.cy = (res & 0xffff0000) > 0;

      // println!("h: {:01$x}", cpu_state.h, 2);
      // println!("l: {:01$x}", cpu_state.l, 2);

      operation_cycles = 10; 

      // panic!("break");
    }, 

    //DCR C  decrement single u8 register C 5c; os=1
    0x0d => {  
      let res: u8 = (Wrapping(cpu_state.c) - Wrapping(1)).0;
      cpu_state.cc.z = res == 0;
      cpu_state.cc.s = 0x80 == (res & 0x80);
      cpu_state.cc.p = parity(res, 8);
      cpu_state.c = res;

      operation_cycles = 5;
    },

    //MVI C, u8 move immediate to C; 7c; os=2byte
    0x0e => { cpu_state.c = operation_arg1; cpu_state.pc += 1; operation_cycles = 7;},

    //LXI D, u16 ; 10c; os=3byte  load intermediate to combined register DE (just called D as 16 bit register)  
    0x11 => { cpu_state.e = operation_arg1; cpu_state.d = operation_arg2; cpu_state.pc += 2; operation_cycles = 10; },

    //INX D ;5c; os=1byte increment register DE
    0x13 => { 
      cpu_state.e = (Wrapping(cpu_state.e) + Wrapping(1)).0;
      if cpu_state.e == 0 {
        cpu_state.d = (Wrapping(cpu_state.d) + Wrapping(1)).0;
      }

      operation_cycles = 5;
    },

    //DAD D  add register pair DE to HL  ; 10c, ox=1
    0x19 => {  
      let de: u32 = (cpu_state.d as u32) << 8 | cpu_state.e as u32;
      let hl: u32 = (cpu_state.h as u32) << 8 | cpu_state.l as u32;

      // println!("de: {:01$x}", de, 4);
      // println!("hl: {:01$x}", hl, 4);

      let res: u32 = de + hl;

      // println!("res: {:01$x}", res, 4);

      cpu_state.h = ((res & 0xff00) >> 8) as u8;
      cpu_state.l = (res & 0xff) as u8;
      cpu_state.cc.cy = (res & 0xffff0000) != 0;

      // println!("h: {:01$x}", cpu_state.h, 2);
      // println!("l: {:01$x}", cpu_state.l, 2);

      operation_cycles = 10; 

      // panic!("break");
    }, 

    //NOP ;4c ;os=1byte
    0x20 => { operation_cycles = 4; },

    //LDAX D load ; 7c; os=1byte; load memory indirect from combinded register DE (just called D as 16 bit register) to A 
    0x1a => { 
      let memory_offset: u16 = ((cpu_state.d as u16) << 8) | (cpu_state.e as u16); 
      // println!("{:01$x}", memory_offset, 4);
      // println!("{:01$x}", cpu_state.memory[memory_offset as usize], 2);
      cpu_state.a = cpu_state.memory[memory_offset as usize];
      operation_cycles = 7; 
    },

    //MVI H, u8 move immediate to H; 7c; os=2byte
    0x26 => { cpu_state.h = operation_arg1; cpu_state.pc += 1; operation_cycles = 7;},

    //LXI H, u16 ; 10c; os=3byte  load intermediate to combined register HL (just called H as 16 bit register)
    0x21 => { cpu_state.l = operation_arg1; cpu_state.h = operation_arg2; cpu_state.pc += 2; operation_cycles = 10; },

    //INX H ;5c; os=1byte  increment register HL
    0x23 => { 
      cpu_state.l = (Wrapping(cpu_state.l) + Wrapping(1)).0 ; 
      if cpu_state.l == 0 {
        cpu_state.h = (Wrapping(cpu_state.h) + Wrapping(1)).0;
      }

      operation_cycles = 5;
    },

    //DAD H  add register pair HL to HL (HLx2) ; 10c, ox=1
    0x29 => {  
      let hl: u32 = (cpu_state.h as u32) << 8 | cpu_state.l as u32;
      let res: u32 = hl + hl;
      cpu_state.h = ((res & 0xff00) >> 8) as u8;
      cpu_state.l = (res & 0xff) as u8;
      cpu_state.cc.cy = (res & 0xffff0000) != 0;

      operation_cycles = 10; 
    }, 

    //LXI sp, u16   Load registerpair u16 immediate to stack pointer(which is u16) ;10c ;os=3byte
    0x31 => { cpu_state.sp = (operation_arg2 as u16) << 8 | (operation_arg1 as u16); cpu_state.pc += 2; operation_cycles = 10; },

    //MVI M,byte move immediate memory; 10c; os=2byte 
    0x36 => {
      let offset: u16 = ((cpu_state.h as u16) << 8) | cpu_state.l as u16;
      cpu_state.memory[offset as usize] = operation_arg1;
      cpu_state.pc += 1;

      operation_cycles = 10;
    },

    //LDA  load register A direct
    0x3a => {
      let offset: u16 = (operation_arg2 as u16) << 8 | operation_arg1 as u16;
      cpu_state.a = cpu_state.memory[offset as usize];
      cpu_state.pc += 2;// panic!("break");

      operation_cycles = 13;
    }

    //MOV L,A ; move from register A to L; 5c; os=1
    0x6f => { cpu_state.l = cpu_state.a; operation_cycles = 5;}

    //MOV M,A ; 7c; os=1  move content of register A to memory location pointed by register HL 16 bit ...
    0x77 => {  
      let memory_offset: u16 = ((cpu_state.h as u16) << 8) | (cpu_state.l as u16);
      cpu_state.memory[memory_offset as usize] = cpu_state.a;
      // println!("{:01$x}", memory_offset, 4);
      // println!("{:01$x}", cpu_state.memory[memory_offset as usize], 2);
      operation_cycles = 7; 
    },

    //MOV H,A ; 5c; os=1byte
    0x7c => { cpu_state.a = cpu_state.h; operation_cycles = 5; }, 

    //POP B  pop register pair BC 10c; os=1;
    0xc1 => {  
      cpu_state.c = cpu_state.memory[cpu_state.sp as usize];
      cpu_state.b = cpu_state.memory[(cpu_state.sp + 1) as usize];
      cpu_state.sp += 2;

      operation_cycles = 10;
    },

    //JNZ adress u16 Jump on none zero ; 10c; os=3byte  
    0xc2 => { 
      if !cpu_state.cc.z {
        cpu_state.pc = (operation_arg2 as u16) << 8 | (operation_arg1 as u16);
      } else {
        cpu_state.pc += 2;
      }

      operation_cycles = 10;
    },

    //JMP u16  jump to u16 adress ;10c ; os=3byte
    0xc3 => { cpu_state.pc = (operation_arg2 as u16) << 8 | (operation_arg1 as u16); operation_cycles = 10; },

    //PUSH B   push the register pair BC an the stack  11c; os=1
    0xc5 => {  
      cpu_state.memory[(cpu_state.sp - 1) as usize] = cpu_state.b;
      cpu_state.memory[(cpu_state.sp - 2) as usize] = cpu_state.c;
      cpu_state.sp -= 2;

      operation_cycles = 11; 
    },

    //RET
    0xc9 => {  
      //load return adress from stack in to program counter
      cpu_state.pc = cpu_state.memory[cpu_state.sp as usize] as u16 | ((cpu_state.memory[(cpu_state.sp + 1) as usize] as u16) << 8);
      cpu_state.sp += 2; // remove address from stack
    },

    //POP D  pop register pair DE 10c; os=1;
    0xd1 => {  
      cpu_state.e = cpu_state.memory[cpu_state.sp as usize];
      cpu_state.d = cpu_state.memory[(cpu_state.sp + 1) as usize];
      cpu_state.sp += 2;

      operation_cycles = 10;
    },

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

    //OUT u8  .. outputs the contend of register A to specified data port ... skip for now
    0xd3 => { cpu_state.pc += 1; }

    //PUSH D push register pair DE to stack; 11c; os=1byte
    0xd5 => { 
      cpu_state.memory[(cpu_state.sp - 1) as usize] = cpu_state.d;
      cpu_state.memory[(cpu_state.sp - 2) as usize] = cpu_state.e;
      cpu_state.sp -= 2;

      operation_cycles = 11; 
    },

    //POP H  pop register pair HL 10c; os=1;
    0xe1 => {  
      cpu_state.l = cpu_state.memory[cpu_state.sp as usize];
      cpu_state.h = cpu_state.memory[(cpu_state.sp + 1) as usize];
      cpu_state.sp += 2;

      operation_cycles = 10;
    },

    //PUSH H push register pair HL to stack; 11c; os=1byte
    0xe5 => { 
      cpu_state.memory[(cpu_state.sp - 1) as usize] = cpu_state.h;
      cpu_state.memory[(cpu_state.sp - 2) as usize] = cpu_state.l;
      cpu_state.sp -= 2;

      operation_cycles = 11; 
    },

    //XCHG   exchange register pairs DE <-> HL 5c; os=1
    0xeb => {  
      let d: u8 = cpu_state.d;
      let e: u8 = cpu_state.e;
      cpu_state.d = cpu_state.h;
      cpu_state.e = cpu_state.l;
      cpu_state.h = d;
      cpu_state.l = e;

      operation_cycles = 5;
    },

    //CPI byte compare immediate with A ;7c ; os=2byte
    0xfe => {  
      let cpm: u8 = (Wrapping(cpu_state.a) - Wrapping(operation_arg1)).0;
      cpu_state.cc.z = cpm == 0;
      cpu_state.cc.s = 0x80 == ( cpm & 0x80 );
      cpu_state.cc.p = parity(cpm, 8);
      cpu_state.cc.cy = cpu_state.a < operation_arg1;
      cpu_state.pc += 1;

      operation_cycles = 7; 
    },
    
    _ => panic!("the opcode: {:01$x} is not yet implemented, shutting down vm.", operation_code, 2),
  }
  println!("z:{:?} s:{:?} p:{:?} cy:{:?} ac:{:?}",cpu_state.cc.z, cpu_state.cc.s, cpu_state.cc.p, cpu_state.cc.cy, cpu_state.cc.ac );
  println!("A:{:09$x} B:{:09$x} C:{:09$x} D:{:09$x} E:{:09$x} H:{:09$x} L:{:09$x} SP:{:010$x} PC:{:010$x}", cpu_state.a, cpu_state.b, cpu_state.c, cpu_state.d, cpu_state.e, cpu_state.h, cpu_state.l, cpu_state.sp, cpu_state.pc, 2, 4);
  // println!("Stack u16:{:01$x}", cpu_state.memory[cpu_state.sp as usize] as u16 | ((cpu_state.memory[(cpu_state.sp + 1) as usize] as u16) << 8), 4);
  println!("\n");
  return 0;
}

#[test]
fn parity_test() {

  assert_eq!(parity(0u8, 8), true); // zero is even .. ?
  assert_eq!(parity(1u8, 8), false);
  assert_eq!(parity(2u8, 8), false);
  assert_eq!(parity(3u8, 8), true);
  assert_eq!(parity(4u8, 8), false);
  assert_eq!(parity(5u8, 8), true);
  assert_eq!(parity(6u8, 8), true);
  assert_eq!(parity(7u8, 8), false);
  assert_eq!(parity(8u8, 8), false);
  assert_eq!(parity(9u8, 8), true);

}

/// counts the number of 1 in binary format
fn parity(_x: u8, size: usize) -> bool {
  let mut p = 0;      //number of ones
  let mut x = _x;

  for i in 0..size {  // count every diget if its a one
    if 1 == (x & 0x1) { p += 1; }
    x = x >> 1;
  }

  0 == (p & 0x1)      // true if the count is even
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
    0x09 => { write!(&mut output, "DAD \tB\n");},
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
    0x26 => { write!(&mut output, "MVI \tH #${:01$x}\n", operation_arg1, 2); operation_size = 2},
    0x27 => { write!(&mut output, "DAA \n"); },
    0x29 => { write!(&mut output, "DAD \tH \n"); },
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
    0x7c => { write!(&mut output, "MOV \tA, H\n");},
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