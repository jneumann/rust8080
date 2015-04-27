#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

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
    z: u8,
    s: u8,
    p: u8,
    cy: u8,
    ac: u8,
    pad: u8,
}

struct CpuState {
    a: u8,
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
  let mut cpuState = initCpu();

  println!("{:?}", &cpuState.a);

  let mut done: i32 = 0;

  while(done == 0) {
  	println!("emulate");
  	done = emulate(&mut cpuState);
  }


}

fn initCpu() -> CpuState {

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

fn emulate(mut cpu_state: &mut CpuState) -> i32 {

	println!("run emulator");

	if cpu_state.pc == 0x2000 {
		println!("no more code to execute");
		return 1;
	}

	println!("code left");

	let operation_code = cpu_state.memory[cpu_state.pc as usize];

	match operation_code {
		//NOP
		0x00 => { println!("NOP"); },
		_ => panic!("the opcode: {} is not jet implemented, shutting down vm.", operation_code),
	}

	cpu_state.pc+=1;

	return 0;
}
