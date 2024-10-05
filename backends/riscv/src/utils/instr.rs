use std::fmt::Debug;

pub struct RTypeInstr {
    instr: u32,
}

impl RTypeInstr {
    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        RTypeInstr {
            instr: u32::from_le_bytes(*bytes),
        }
    }

    pub fn builder() -> RTypeBuilder {
        RTypeBuilder::default()
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.instr.to_le_bytes()
    }

    pub fn opcode(&self) -> u8 {
        (self.instr & 0b1111111) as u8
    }

    pub fn rd(&self) -> u8 {
        ((self.instr & (0b11111 << 7)) >> 7) as u8
    }

    pub fn funct3(&self) -> u8 {
        ((self.instr & (0b111 << 12)) >> 12) as u8
    }

    pub fn rs1(&self) -> u8 {
        ((self.instr & (0b11111 << 15)) >> 15) as u8
    }

    pub fn rs2(&self) -> u8 {
        ((self.instr & (0b11111 << 20)) >> 20) as u8
    }

    pub fn funct7(&self) -> u8 {
        ((self.instr & (0b1111111 << 25)) >> 25) as u8
    }
}

#[derive(Default)]
pub struct RTypeBuilder {
    instr: u32,
}

impl RTypeBuilder {
    pub fn opcode(mut self, opcode: u8) -> Self {
        assert!(opcode <= 0b1111111);
        self.instr += opcode as u32;
        self
    }

    pub fn rd(mut self, rd: u8) -> Self {
        assert!(rd <= 0b11111);
        self.instr += (rd as u32) << 7;
        self
    }

    pub fn funct3(mut self, funct3: u8) -> Self {
        assert!(funct3 <= 0b111);
        self.instr += (funct3 as u32) << 12;
        self
    }

    pub fn rs1(mut self, rs1: u8) -> Self {
        assert!(rs1 <= 0b11111);
        self.instr += (rs1 as u32) << 15;
        self
    }

    pub fn rs2(mut self, rs2: u8) -> Self {
        assert!(rs2 <= 0b11111);
        self.instr += (rs2 as u32) << 20;
        self
    }

    pub fn funct7(mut self, funct7: u8) -> Self {
        assert!(funct7 <= 0b1111111);
        self.instr += (funct7 as u32) << 25;
        self
    }

    pub fn build(self) -> RTypeInstr {
        RTypeInstr { instr: self.instr }
    }
}

impl Debug for RTypeInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("{:#032b}: opcode = {:#07b}, rd = {:#05b}, funct3 = {:#03b}, rs1 = {:#05b}, rs2 = {:#05b}, funct7 = {:#07b}", self.instr, self.opcode(), self.rd(), self.funct3(), self.rs1(), self.rs2(), self.funct7());
        f.write_str(&string)
    }
}

pub struct ITypeInstr {
    instr: u32,
}

impl ITypeInstr {
    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        ITypeInstr {
            instr: u32::from_le_bytes(*bytes),
        }
    }

    pub fn builder() -> ITypeBuilder {
        ITypeBuilder::default()
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.instr.to_le_bytes()
    }

    pub fn opcode(&self) -> u8 {
        (self.instr & 0b1111111) as u8
    }

    pub fn rd(&self) -> u8 {
        ((self.instr & (0b11111 << 7)) >> 7) as u8
    }

    pub fn funct3(&self) -> u8 {
        ((self.instr & (0b111 << 12)) >> 12) as u8
    }

    pub fn rs1(&self) -> u8 {
        ((self.instr & (0b11111 << 15)) >> 15) as u8
    }

    pub fn imm(&self) -> i16 {
        let num = ((self.instr & (0b11111111111 << 19)) >> 19) as i16;
        let sign = (self.instr & (0b1 << 31)) >> 31;
        if sign == 0 {
            num
        } else {
            -num
        }
    }
}

impl Debug for ITypeInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("{:#032b}: opcode = {:#07b}, rd = {:#05b}, funct3 = {:#03b}, rs1 = {:#05b}, imm = {:#12b}", self.instr, self.opcode(), self.rd(), self.funct3(), self.rs1(), self.imm());
        f.write_str(&string)
    }
}

#[derive(Default)]
pub struct ITypeBuilder {
    instr: u32,
}

impl ITypeBuilder {
    pub fn opcode(mut self, opcode: u8) -> Self {
        assert!(opcode <= 0b1111111);
        self.instr += opcode as u32;
        self
    }

    pub fn rd(mut self, rd: u8) -> Self {
        assert!(rd <= 0b11111);
        self.instr += (rd as u32) << 7;
        self
    }

    pub fn funct3(mut self, funct3: u8) -> Self {
        assert!(funct3 <= 0b111);
        self.instr += (funct3 as u32) << 12;
        self
    }

    pub fn rs1(mut self, rs1: u8) -> Self {
        assert!(rs1 <= 0b11111);
        self.instr += (rs1 as u32) << 15;
        self
    }

    pub fn imm(mut self, imm: i16) -> Self {
        let num = imm.abs();
        assert!(num <= 0b11111111111);
        let sign = if imm < 0 { 1 } else { 0 };
        self.instr += (num as u32) << 19;
        self.instr += (sign as u32) << 31;
        self
    }

    pub fn build(self) -> ITypeInstr {
        ITypeInstr { instr: self.instr }
    }
}

pub struct STypeInstr {
    instr: u32,
}

impl STypeInstr {
    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        STypeInstr {
            instr: u32::from_le_bytes(*bytes),
        }
    }

    pub fn builder() -> STypeBuilder {
        STypeBuilder::default()
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.instr.to_le_bytes()
    }

    pub fn opcode(&self) -> u8 {
        (self.instr & 0b1111111) as u8
    }

    pub fn funct3(&self) -> u8 {
        ((self.instr & (0b111 << 12)) >> 12) as u8
    }

    pub fn rs1(&self) -> u8 {
        ((self.instr & (0b11111 << 15)) >> 15) as u8
    }

    pub fn rs2(&self) -> u8 {
        ((self.instr & (0b11111 << 20)) >> 20) as u8
    }

    pub fn imm(&self) -> i16 {
        let part1 = ((self.instr & (0b11111 << 6)) >> 6) as i16;
        let part2 = ((self.instr & (0b111111 << 24)) >> 19) as i16;
        let sign = (self.instr & (0b1 << 31)) >> 31;
        let num = part1 + part2;
        if sign == 0 {
            num
        } else {
            -num
        }
    }
}

impl Debug for STypeInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("{:#032b}: opcode = {:#07b}, funct3 = {:#03b}, rs1 = {:#05b}, rs2 = {:#05b} imm = {:#12b}", self.instr, self.opcode(), self.funct3(), self.rs1(), self.rs2(), self.imm());
        f.write_str(&string)
    }
}

#[derive(Default)]
pub struct STypeBuilder {
    instr: u32,
}

impl STypeBuilder {
    pub fn opcode(mut self, opcode: u8) -> Self {
        assert!(opcode <= 0b1111111);
        self.instr += opcode as u32;
        self
    }

    pub fn funct3(mut self, funct3: u8) -> Self {
        assert!(funct3 <= 0b111);
        self.instr += (funct3 as u32) << 12;
        self
    }

    pub fn rs1(mut self, rs1: u8) -> Self {
        assert!(rs1 <= 0b11111);
        self.instr += (rs1 as u32) << 15;
        self
    }

    pub fn rs2(mut self, rs2: u8) -> Self {
        assert!(rs2 <= 0b11111);
        self.instr += (rs2 as u32) << 20;
        self
    }

    pub fn imm(mut self, imm: i16) -> Self {
        let part1 = (imm & 0b11111) as u32;
        let part2 = ((imm & (0b111111 << 5)) >> 5) as u32;
        let sign = if imm < 0 { 1 } else { 0 };
        self.instr += part1 << 6;
        self.instr += part2 << 24;
        self.instr += (sign as u32) << 31;
        self
    }

    pub fn build(self) -> STypeInstr {
        STypeInstr { instr: self.instr }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::RTypeInstr;

    #[test]
    fn test_rtype() {
        let word: u32 = 7537331;
        let bytes = word.to_le_bytes();

        let instr = RTypeInstr::from_bytes(&bytes);

        println!("{:?}", instr);
        assert_eq!(instr.opcode(), 0b0110011_u8);
        assert_eq!(instr.rd(), 5);
        assert_eq!(instr.funct3(), 0);
        assert_eq!(instr.rs1(), 6);
        assert_eq!(instr.rs2(), 7);
        assert_eq!(instr.funct7(), 0);
    }

    #[test]
    fn test_rtype_builder() {
        let instr = RTypeInstr::builder()
            .opcode(0b11111)
            .funct3(0b111)
            .rd(0b11011)
            .rs1(0b00001)
            .rs2(0b11000)
            .funct7(0b1100000)
            .build();

        assert_eq!(instr.opcode(), 0b11111);
        assert_eq!(instr.funct3(), 0b111);
        assert_eq!(instr.rd(), 0b11011);
        assert_eq!(instr.rs1(), 0b00001);
        assert_eq!(instr.rs2(), 0b11000);
        assert_eq!(instr.funct7(), 0b1100000);
    }
}
