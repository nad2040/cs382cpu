use crate::parser::{
    Instruction, RegImmAddr, CONSTANT_POOL_OFFSET, DATA_OFFSET, FILE_LIMIT, TEXT_OFFSET,
};
use log::error;

pub fn generate_files(
    program: String,
    constant_pool: &Vec<u8>,
    data: &Vec<u8>,
    instructions: &Vec<Instruction>,
) {
    let header: String = "v3.0 hex words addressed\n".to_string();
    let mut data_file: String = header.clone();

    // debug!("{:?}", constant_pool);
    // debug!("{:?}", data);
    // debug!("{:?}", instructions);

    for i in CONSTANT_POOL_OFFSET..DATA_OFFSET {
        // line header
        if i % 16 == 0 {
            data_file.push_str(format!("{:04x}: ", i).as_str());
        }

        if i < constant_pool.len() {
            // Get the translation into 1 byte in hex for that
            data_file.push_str(format!("{:02x}", constant_pool[i]).as_str());
        } else {
            data_file.push_str("00");
        }

        if i % 16 == 15 {
            data_file.push('\n');
        } else {
            data_file.push(' ');
        }
    }

    for i in DATA_OFFSET..TEXT_OFFSET {
        let index = i - DATA_OFFSET;
        // line header
        if i % 16 == 0 {
            data_file.push_str(format!("{:04x}: ", i).as_str());
        }

        if index < data.len() {
            // Get the translation into 1 byte in hex for that
            data_file.push_str(format!("{:02x}", data[index]).as_str());
        } else {
            data_file.push_str("00");
        }

        if i % 16 == 15 {
            data_file.push('\n');
        } else {
            data_file.push(' ');
        }
    }

    match std::fs::write(format!("{}_data_section.txt", program).as_str(), data_file) {
        Ok(_) => (),
        Err(_) => {
            error!("Couldn't generate data section file.");
            std::process::exit(1);
        }
    }

    let mut text_file: String = header.clone();
    let encoded_instructions = encode_instructions(instructions);
    for i in TEXT_OFFSET..FILE_LIMIT {
        let index = i - TEXT_OFFSET;
        // line header
        if i % 16 == 0 {
            text_file.push_str(format!("{:04x}: ", index).as_str());
        }

        if index < encoded_instructions.len() {
            // Get the translation into 1 byte in hex for that
            text_file.push_str(format!("{:02x}", encoded_instructions[index]).as_str());
        } else {
            text_file.push_str("00");
        }

        if i % 16 == 15 {
            text_file.push('\n');
        } else {
            text_file.push(' ');
        }
    }

    match std::fs::write(format!("{}_text_section.txt", program).as_str(), text_file) {
        Ok(_) => (),
        Err(_) => {
            error!("Couldn't generate text section file.");
            std::process::exit(1);
        }
    }
}

fn encode_instructions(instructions: &Vec<Instruction>) -> Vec<u8> {
    let mut encoded_instructions: Vec<u8> = Vec::new();
    for i in 0..instructions.len() {
        let mut opcode: u32 = 0b0000000;
        let mut imm: u32 = 0x0000;
        let mut rm: u32 = 0b000;
        let mut rn: u32 = 0b000;
        let mut rd: u32 = 0b000;
        match &instructions[i] {
            Instruction::Halt => (),
            Instruction::Add(dst, src1, src2) => {
                opcode |= 0b1000000;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Sub(dst, src1, src2) => {
                opcode |= 0b1000100;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Mul(dst, src1, src2) => {
                opcode |= 0b1001000;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Div(dst, src1, src2) => {
                opcode |= 0b1001100;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Mod(dst, src1, src2) => {
                opcode |= 0b1010000;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Asr(dst, src1, src2) => {
                opcode |= 0b1010100;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Lsl(dst, src1, src2) => {
                opcode |= 0b1011000;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::And(dst, src1, src2) => {
                opcode |= 0b1011100;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Orr(dst, src1, src2) => {
                opcode |= 0b1100000;
                rd |= *dst as u32;
                rn |= *src1 as u32;
                match src2 {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rm |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Neg(dst, src) => {
                opcode |= 0b1100100;
                rd |= *dst as u32;
                match src {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src2) => {
                        rn |= *src2 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::Swap(reg1, reg2) => {
                opcode |= 0b1101101;
                rd |= *reg1 as u32;
                rn |= *reg2 as u32;
            }
            Instruction::Ld(dst, src) => {
                opcode |= 0b1101000;
                rd |= *dst as u32;
                match src {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(src) => {
                        rn |= *src as u32;
                    }
                    RegImmAddr::Address(addr) => {
                        opcode |= 0b0000010;
                        imm |= *addr as u16 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::LdMem(num_bytes, sign_extend, dst, addr_reg, offset) => {
                match *num_bytes {
                    1 => opcode |= 0b0000100,
                    2 => opcode |= 0b0001000,
                    4 => opcode |= 0b0001100,
                    8 => opcode |= 0b0010000,
                    _ => (),
                }
                if *sign_extend {
                    opcode |= 0b0010000;
                }
                rd |= *dst as u32;
                rn |= *addr_reg as u32;
                match offset {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(offset_reg) => {
                        rm |= *offset_reg as u32;
                    }
                    _ => (),
                }
            }
            Instruction::St(num_bytes, src, addr_reg, offset) => {
                match *num_bytes {
                    1 => opcode |= 0b0100000,
                    2 => opcode |= 0b0100100,
                    4 => opcode |= 0b0101000,
                    8 => opcode |= 0b0101100,
                    _ => (),
                }
                rd |= *src as u32;
                rn |= *addr_reg as u32;
                match offset {
                    RegImmAddr::Imm(num) => {
                        opcode |= 0b0000010;
                        imm |= *num as u16 as u32;
                    }
                    RegImmAddr::Register(offset_reg) => {
                        rm |= *offset_reg as u32;
                    }
                    _ => (),
                }
            }
            Instruction::B(addr) => {
                opcode |= 0b1110000;
                match addr {
                    RegImmAddr::Address(addr) => {
                        imm |= *addr as u16 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::CBZ(reg, addr) => {
                opcode |= 0b1110100;
                rn |= *reg as u32;
                match addr {
                    RegImmAddr::Address(addr) => {
                        imm |= *addr as u16 as u32;
                    }
                    _ => (),
                }
            }
            Instruction::CBNZ(reg, addr) => {
                opcode |= 0b1111000;
                rn |= *reg as u32;
                match addr {
                    RegImmAddr::Address(addr) => {
                        imm |= *addr as u16 as u32;
                    }
                    _ => (),
                }
            }
        }
        let instruction = (opcode << 25) | (imm << 9) | (rm << 6) | (rn << 3) | rd;
        encoded_instructions.append(&mut instruction.to_le_bytes().to_vec());
    }
    encoded_instructions
}
