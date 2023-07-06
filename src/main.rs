use assembler::{error::AssemblerError, instructions::Instruction, mnemonics::BranchMnemonic};
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: String,
}

const INST_SIZE: u32 = 0x20;

// TODO: this should probably go into the linker
fn preprocess_asm_file(filename: &str) -> Result<(Vec<String>, HashSet<String>), AssemblerError> {
    let file = File::open(filename)?;

    let mut label_map = HashMap::new();
    let mut pc = 0x00;
    let mut program = vec![];
    let mut preprocessed_program = vec![];
    let mut strings = HashSet::new();

    // XXX: this is good enough for now but is probably inaccurate for some cases
    let is_label = |l: &str| -> bool { l.ends_with(':') };

    // Read in the file, figure out the mapping of label -> pc
    for line in BufReader::new(&file).lines() {
        let l = line?;
        // Get the addr of the instruction following the label, but don't write the label to the
        // program buffer since we don't need them for assembling
        if is_label(&l) {
            let label = l.replace(':', "");
            label_map.insert(label.to_owned(), pc);
            strings.insert(label.to_owned());
        } else {
            pc += INST_SIZE;
        }

        program.push((pc, l.to_owned()));
    }

    // Make a second pass now that we know where all the labels are located.
    for (pc, inst) in program {
        let modified_inst = if let Ok(_) = BranchMnemonic::try_from(inst.as_str()) {
            let (_mnemonic, label) = inst.trim().split_once(' ').unwrap();
            let label_addr = label_map.get(label).unwrap();
            inst.replace(label, &(label_addr - pc).to_string())
                .trim()
                .to_string()
        } else {
            inst.trim().to_owned()
        };

        preprocessed_program.push(modified_inst);
    }

    Ok((preprocessed_program, strings))
}

fn main() -> Result<(), AssemblerError> {
    let args = Args::parse();
    let (preprocessed_program, strings) = preprocess_asm_file(&args.file)?;

    for inst in &preprocessed_program {
        // Ignore labels and directives
        if let Ok(parsed_instruction) = Instruction::try_from(inst.as_str()) {
            // dbg!(&parsed_instruction);
            let machine_code: u32 = parsed_instruction.to_machine_code();
            println!("{inst} - {machine_code:08x}")
        }
    }

    Ok(())
}
