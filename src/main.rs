mod asm;

use std::{io::Write, time::Instant};

use asm::assemble;
use bitint::prelude::*;
use bytemuck::Pod;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToBytes, ToPrimitive};

pub type Address = u16;
pub const PC: Address = 0x0;
pub const WRITING: Address = 0xFFFE;
pub const DATA: Address = 0xFFFF;

#[derive(FromPrimitive, ToPrimitive)]
pub enum Opcodes {
    ZeroPageAdd = 0b0000_0000,
    ZeroPageNeg = 0b0000_0001,
    ZeroPageAnd = 0b0000_0010,
    ZeroPageOr = 0b0000_0100,
    ZeroPageXor = 0b0000_0101,
    ZeroPageLoad = 0b0000_0110,
    ZeroPageStore = 0b0000_1001,
    ZeroPageImmediateLoad = 0b0000_1010,
    ZeroPageLoadIfPos = 0b0000_1100,
}

pub trait Inst<const N: usize> {
    fn from_bytes(bytes: &[u32; N]) -> Self;
    fn to_bytes(&self) -> [u32; N];
}

#[derive(Clone, Copy, Debug)]
pub struct ZeroPageAdd {
    lhs: u8,
    rhs: u8,
    out: u8,
}

impl Inst<1> for ZeroPageAdd {
    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageAdd as u8,
            self.lhs,
            self.rhs,
            self.out,
        ])]
    }

    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageAdd as u8);
        Self {
            lhs: bytes[1],
            rhs: bytes[2],
            out: bytes[3],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZeroPageAnd {
    lhs: u8,
    rhs: u8,
    out: u8,
}

impl Inst<1> for ZeroPageAnd {
    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageAnd as u8,
            self.lhs,
            self.rhs,
            self.out,
        ])]
    }

    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageAnd as u8);
        Self {
            lhs: bytes[1],
            rhs: bytes[2],
            out: bytes[3],
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ZeroPageOr {
    lhs: u8,
    rhs: u8,
    out: u8,
}

impl Inst<1> for ZeroPageOr {
    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageOr as u8,
            self.lhs,
            self.rhs,
            self.out,
        ])]
    }

    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageOr as u8);
        Self {
            lhs: bytes[1],
            rhs: bytes[2],
            out: bytes[3],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZeroPageXor {
    lhs: u8,
    rhs: u8,
    out: u8,
}

impl Inst<1> for ZeroPageXor {
    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageXor as u8,
            self.lhs,
            self.rhs,
            self.out,
        ])]
    }

    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageXor as u8);
        Self {
            lhs: bytes[1],
            rhs: bytes[2],
            out: bytes[3],
        }
    }
}

pub struct ZeroPageNegate {
    input: u8,
    out: u8,
}

impl Inst<1> for ZeroPageNegate {
    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageNeg as u8);
        Self {
            input: bytes[1],
            out: bytes[2],
        }
    }

    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageNeg as u8,
            self.input,
            self.out,
            0,
        ])]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZeroPageImmediateLoad {
    addr: u8,
    imm: u16,
}

impl Inst<1> for ZeroPageImmediateLoad {
    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageImmediateLoad as u8);
        Self {
            addr: bytes[1],
            imm: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }

    fn to_bytes(&self) -> [u32; 1] {
        let imm = self.imm.to_le_bytes();
        [u32::from_be_bytes([
            Opcodes::ZeroPageImmediateLoad as u8,
            self.addr,
            imm[0],
            imm[1],
        ])]
    }
}

pub struct ZeroPageLoad {
    from: u16,
    to: u8,
}

impl Inst<1> for ZeroPageLoad {
    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageLoad as u8);
        Self {
            from: u16::from_le_bytes([bytes[1], bytes[2]]),
            to: bytes[3],
        }
    }

    fn to_bytes(&self) -> [u32; 1] {
        let from = self.from.to_le_bytes();
        [u32::from_be_bytes([
            Opcodes::ZeroPageLoad as u8,
            from[0],
            from[1],
            self.to,
        ])]
    }
}

pub struct ZeroPageStore {
    from: u8,
    to: u16,
}

impl Inst<1> for ZeroPageStore {
    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageStore as u8);
        Self {
            from: bytes[1],
            to: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }

    fn to_bytes(&self) -> [u32; 1] {
        let to = self.to.to_le_bytes();
        [u32::from_be_bytes([
            Opcodes::ZeroPageStore as u8,
            self.from,
            to[0],
            to[1],
        ])]
    }
}

pub struct ZeroPageLoadIfPos {
    cond: u8,
    from: u8,
    to: u8,
}

impl Inst<1> for ZeroPageLoadIfPos {
    fn from_bytes(bytes: &[u32; 1]) -> Self {
        let bytes = bytes[0].to_be_bytes();
        assert_eq!(bytes[0], Opcodes::ZeroPageLoadIfPos as u8);
        Self {
            cond: bytes[1],
            from: bytes[2],
            to: bytes[3],
        }
    }

    fn to_bytes(&self) -> [u32; 1] {
        [u32::from_be_bytes([
            Opcodes::ZeroPageLoadIfPos as u8,
            self.cond,
            self.from,
            self.to,
        ])]
    }
}

#[derive(Clone, Debug)]
pub struct Machine {
    memory: [u32; 2_usize.pow(16)],
    breakpoints: Vec<Address>,
}

impl Machine {
    pub fn read(&self, addr: Address) -> u32 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: Address, value: u32) {
        self.memory[addr as usize] = value
    }

    pub fn read_n(&self, addr: Address, len: usize) -> &[u32] {
        &self.memory[addr as usize..addr as usize + len]
    }

    pub fn write_n(&mut self, addr: Address, data: &[u32]) {
        self.memory[addr as usize..addr as usize + data.len()].copy_from_slice(data)
    }

    pub fn read_bytes<const N: usize>(&self, addr: Address) -> &[u32; N] {
        self.memory[addr as usize..addr as usize + N]
            .try_into()
            .unwrap()
    }

    pub fn write_bytes<const N: usize>(&mut self, addr: Address, data: &[u32; N]) {
        self.memory[addr as usize..addr as usize + N].copy_from_slice(data)
    }

    pub fn program(mut self, program: &[u32], addr: Address) -> Self {
        self.memory[addr as usize..addr as usize + program.len()].copy_from_slice(program);

        self
    }

    pub fn breakpoint(mut self, addr: Address) -> Self {
        self.breakpoints.push(addr);

        self
    }

    pub fn set(mut self, addr: Address, value: u32) -> Self {
        self.write(addr, value);

        self
    }

    pub fn run(mut self) -> Self {
        let mut start = Instant::now();
        let mut i = 0;
        loop {
            let pc: Address = self.read(PC) as u16;

            if self.breakpoints.contains(&pc) {
                self.debug()
            }

            let inst = self.read(pc);
            let bytes = inst.to_be_bytes();
            let opcode = bytes[0];
            //println!("{:08b}", opcode);
            let length = if (opcode & 0b11) == 0b11 {
                // VLE
                todo!("VLE is not yet supported")
            } else {
                1
            };
            let inst = self.read_n(pc, length);

            match Opcodes::from_u8(opcode).unwrap() {
                Opcodes::ZeroPageAdd => {
                    let inst = ZeroPageAdd::from_bytes(inst.try_into().unwrap());
                    self.write(
                        inst.out as Address,
                        self.read(inst.lhs as Address) + self.read(inst.rhs as Address),
                    );
                }
                Opcodes::ZeroPageNeg => {
                    let inst = ZeroPageNegate::from_bytes(inst.try_into().unwrap());
                    self.write(
                        inst.out as Address,
                        bytemuck::cast::<i32, u32>(
                            -(bytemuck::cast::<u32, i32>(self.read(inst.input as Address))),
                        ),
                    )
                }
                Opcodes::ZeroPageAnd => {
                    let inst = ZeroPageAnd::from_bytes(inst.try_into().unwrap());
                    self.write(
                        inst.out as Address,
                        self.read(inst.lhs as Address) & self.read(inst.rhs as Address),
                    );
                }
                Opcodes::ZeroPageOr => {
                    let inst = ZeroPageOr::from_bytes(inst.try_into().unwrap());
                    self.write(
                        inst.out as Address,
                        self.read(inst.lhs as Address) | self.read(inst.rhs as Address),
                    );
                }
                Opcodes::ZeroPageXor => {
                    let inst = ZeroPageXor::from_bytes(inst.try_into().unwrap());
                    self.write(
                        inst.out as Address,
                        self.read(inst.lhs as Address) ^ self.read(inst.rhs as Address),
                    );
                }
                Opcodes::ZeroPageImmediateLoad => {
                    let inst = ZeroPageImmediateLoad::from_bytes(inst.try_into().unwrap());
                    self.write(inst.addr as Address, inst.imm as u32);
                }
                Opcodes::ZeroPageLoad => {
                    let inst = ZeroPageLoad::from_bytes(inst.try_into().unwrap());
                    self.write(inst.to as Address, self.read(inst.from as Address))
                }
                Opcodes::ZeroPageStore => {
                    let inst = ZeroPageStore::from_bytes(inst.try_into().unwrap());
                    self.write(inst.to as Address, self.read(inst.from as Address))
                }
                Opcodes::ZeroPageLoadIfPos => {
                    let inst = ZeroPageLoadIfPos::from_bytes(inst.try_into().unwrap());
                    if bytemuck::cast::<u32, i32>(self.read(inst.cond as Address)) > 0 {
                        self.write(inst.to as Address, self.read(inst.from as Address));
                    }
                }
            }

            if self.read(WRITING) != 0 {
                print!("{}", self.read(DATA) as u8 as char);
                std::io::stdout().flush().unwrap();
                self.write(WRITING, 0);
            }

            if pc == self.read(PC) as u16 {
                self.write(PC, pc as u32 + length as u32);
            }

            i += 1;
            if i == 100_000_000 {
                println!("{}MHz", 100. / (Instant::now() - start).as_secs_f64());
                start = Instant::now();
                i = 0;
            }
        }
    }

    pub fn debug(&mut self) {
        println!("Hit breakpoint at 0x{:04x}", self.read(PC));

        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut cmd = String::new();
            std::io::stdin().read_line(&mut cmd).unwrap();
            let cmd = cmd.trim();

            let opcode = cmd.split(' ').nth(0).unwrap();
            match opcode {
                "c" | "cont" | "continue" => break,
                "r" | "read" => println!(
                    "{}",
                    format!(
                        "{:032b}",
                        self.read(
                            Address::from_str_radix(cmd.split(' ').nth(1).unwrap(), 16).unwrap()
                        )
                    )
                    .chars()
                    .enumerate()
                    .map(|(i, c)| c.to_string()
                        + if i % 8 == 7 {
                            "  "
                        } else if i % 4 == 3 {
                            " "
                        } else {
                            ""
                        })
                    .collect::<Vec<String>>()
                    .join("")
                ),
                "exit" => panic!("Exitting"),
                _ => println!("Invalid command"),
            }
        }
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            memory: [0; 2_usize.pow(16)],
            breakpoints: Vec::new(),
        }
    }
}

impl Machine {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct Program {
    program: Vec<u32>,
}

impl Program {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<const N: usize, I: Inst<N>>(mut self, inst: I) -> Self {
        self.program.extend_from_slice(&inst.to_bytes());

        self
    }

    pub fn bytes(self) -> Vec<u32> {
        self.program
    }
}

fn main() {
    print!("Enter assembly path: ");
    std::io::stdout().flush().unwrap();

    let mut path = String::new();
    std::io::stdin().read_line(&mut path).unwrap();
    let (program, breakpoints) =
        assemble(String::from_utf8(std::fs::read(path.trim()).unwrap()).unwrap());

    let machine = Machine::default()
        .set(PC, 0x8000)
        .program(&program.bytes(), 0x8000)
        .breakpoint(0x8000);
    let machine = breakpoints
        .into_iter()
        .fold(machine, |machine, breakpoint| {
            machine.breakpoint(breakpoint)
        });
    machine.run();
}
