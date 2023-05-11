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
    let args = Args::parse();
    let file = File::open(args.file).unwrap();

    for _line in BufReader::new(file).lines() {
        // let parsed_instruction = Instruction::new(&line.unwrap());
        // dbg!(&parsed_instruction);
        // let machine_code: u32 = parsed_instruction.into();
        // println!("{machine_code}")
    }
}
