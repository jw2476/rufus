use std::{char, collections::HashMap};

use crate::{
    Address, Program, ZeroPageAdd, ZeroPageAnd, ZeroPageImmediateLoad, ZeroPageLoad,
    ZeroPageLoadIfPos, ZeroPageNegate, ZeroPageOr, ZeroPageStore, ZeroPageXor,
};

struct ParseState {
    symbols: HashMap<String, Address>,
    program: Program,
    breakpoints: Vec<Address>,
}

fn operand(word: &str, symbols: Option<&HashMap<String, Address>>) -> u64 {
    if word.starts_with("0x") {
        u64::from_str_radix(&word.chars().skip(2).collect::<String>(), 16).unwrap()
    } else if word.starts_with(':') {
        operand(&word.chars().skip(1).collect::<String>(), symbols)
    } else if word.chars().next().unwrap().is_ascii_digit() {
        word.parse().unwrap()
    } else {
        symbols
            .map(|symbols| *symbols.get(word).unwrap())
            .unwrap_or_default() as u64
    }
}

fn add_inst(line: &str, program: Program, symbols: Option<&HashMap<String, Address>>) -> Program {
    let words = line.split(' ');
    let opcode = words.clone().next().unwrap();
    let operand = |i: usize| operand(words.clone().nth(i).unwrap(), symbols);
    match opcode {
        "ADD" => program.push(ZeroPageAdd {
            lhs: operand(1) as u8,
            rhs: operand(2) as u8,
            out: operand(3) as u8,
        }),
        "NEG" => program.push(ZeroPageNegate {
            input: operand(1) as u8,
            out: operand(2) as u8,
        }),
        "AND" => program.push(ZeroPageAnd {
            lhs: operand(1) as u8,
            rhs: operand(2) as u8,
            out: operand(3) as u8,
        }),
        "OR" => program.push(ZeroPageOr {
            lhs: operand(1) as u8,
            rhs: operand(2) as u8,
            out: operand(3) as u8,
        }),
        "XOR" => program.push(ZeroPageXor {
            lhs: operand(1) as u8,
            rhs: operand(2) as u8,
            out: operand(3) as u8,
        }),
        "L" => program.push(ZeroPageLoad {
            from: operand(1) as u16,
            to: operand(2) as u8,
        }),
        "S" => program.push(ZeroPageStore {
            from: operand(1) as u8,
            to: operand(2) as u16,
        }),
        "LI" => program.push(ZeroPageImmediateLoad {
            addr: operand(1) as u8,
            imm: operand(2) as u16,
        }),
        "LP" => program.push(ZeroPageLoadIfPos {
            cond: operand(1) as u8,
            from: operand(2) as u8,
            to: operand(3) as u8,
        }),
        _ => panic!("Unknown opcode {opcode}"),
    }
}

pub fn assemble(asm: String) -> (Program, Vec<Address>) {
    let ParseState {
        symbols,
        breakpoints,
        ..
    } = asm.split('\n').fold(
        ParseState {
            program: Program::new(),
            symbols: HashMap::new(),
            breakpoints: Vec::new(),
        },
        |mut state, line| {
            if line.is_empty() {
                return state;
            }
            let mut line = line.trim().to_string();

            if line.split(' ').next().unwrap().ends_with(':') {
                let mut name = line.split(' ').next().unwrap().chars();
                name.next_back();
                state.symbols.insert(
                    name.as_str().to_string(),
                    0x8000 + state.program.program.len() as Address,
                );
                line = line
                    .split(' ')
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .trim()
                    .to_string();
            }

            if line.starts_with('~') {
                state
                    .breakpoints
                    .push(0x8000 + state.program.program.len() as Address);
                line = line.chars().skip(1).collect::<String>().trim().to_string();
            }

            state.program = add_inst(&line, state.program, None);

            state
        },
    );

    let ParseState { program, .. } = asm.split('\n').fold(
        ParseState {
            program: Program::new(),
            symbols,
            breakpoints: Vec::new(),
        },
        |mut state, line| {
            if line.is_empty() {
                return state;
            }
            let mut line = line.trim().to_string();

            if line.split(' ').next().unwrap().ends_with(':') {
                line = line
                    .split(' ')
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .trim()
                    .to_string();
            }

            if line.starts_with('~') {
                line = line.chars().skip(1).collect::<String>().trim().to_string();
            }

            state.program = add_inst(&line, state.program, Some(&state.symbols));

            state
        },
    );

    (program, breakpoints)
}
