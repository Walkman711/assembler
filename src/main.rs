use assembler::instructions::Instruction;
use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: String,
}

fn main() {
    // let add_inst = Instruction::from("add w8, w0, 1");
    // dbg!(&add_inst);
    // let machine_code: u32 = add_inst.to_machine_code();
    // println!("{:#x}", machine_code);

    // let mov_inst = Instruction::from("mov w0, 1");
    // dbg!(&mov_inst);

    let args = Args::parse();
    let file = File::open(args.file).unwrap();

    for line in BufReader::new(file).lines() {
        let parsed_instruction = Instruction::from(line.unwrap().as_str());
        // dbg!(&parsed_instruction);
        let machine_code: u32 = parsed_instruction.to_machine_code();
        println!("{machine_code:#x}")
    }
}
