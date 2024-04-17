pub enum Endianness {
    Big,
    Little,
}

pub struct TargetOptions {
    pub endiannes: Endianness,
    pub word_size: u8,
    pub flags: Vec<String>,
}
