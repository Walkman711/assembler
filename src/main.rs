use assembler::error::MyErr;
use assembler::instructions::Instruction;
use assembler::mnemonics::BranchMnemonic;
use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: String,
}

const INST_SIZE: u32 = 0x20;

fn main() -> Result<(), MyErr> {
    let args = Args::parse();
    let file = File::open(args.file)?;

    let mut label_map = HashMap::new();
    let mut pc = 0x00;
    let mut program = vec![];

    // XXX: this is good enough for now but is probably inaccurate for some cases
    let is_label = |l: &str| -> bool { l.ends_with(':') };

    // Read in the file, figure out the mapping of label -> pc
    for line in BufReader::new(&file).lines() {
        let l = line?;
        // Get the addr of the instruction following the label, but don't write the label to the
        // program buffer since we don't need them for assembling
        if is_label(&l) {
            label_map.insert(l.replace(':', "").to_owned(), pc);
        } else {
            program.push((pc, l.to_owned()));
            pc += INST_SIZE;
        }
    }

    for (pc, inst) in &program {
        let modified_inst = if let Ok(_) = BranchMnemonic::try_from(inst.as_str()) {
            let (_mnemonic, label) = inst.trim().split_once(' ').unwrap();
            let label_addr = label_map.get(label).unwrap();
            inst.replace(label, &(label_addr - pc).to_string())
                .to_string()
        } else {
            inst.to_owned()
        };

        let parsed_instruction = Instruction::try_from(modified_inst.as_str())?;
        // dbg!(&parsed_instruction);
        let machine_code: u32 = parsed_instruction.to_machine_code();
        println!("{inst} - {machine_code:08x}")
    }

    Ok(())
}
