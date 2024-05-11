pub trait IRFormatter {
    fn increase_indent(&mut self);
    fn decrease_indent(&mut self);
    fn get_indent(&self) -> u32;
    fn write_direct(&mut self, data: &str);

    fn indent(&mut self) {
        for _ in 0..self.get_indent() {
            self.write_direct("  ");
        }
    }

    fn indent_write(&mut self, data: &str) {
        self.indent();
        self.write_direct(data)
    }

    fn start_region(&mut self) {
        self.write_direct("{\n");
        self.increase_indent();
    }

    fn end_region(&mut self) {
        self.decrease_indent();
        self.indent_write("}\n");
    }
}

pub struct StdoutPrinter {
    indent: u32,
}

impl Default for StdoutPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl StdoutPrinter {
    pub fn new() -> Self {
        StdoutPrinter { indent: 0 }
    }
}

impl IRFormatter for StdoutPrinter {
    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    fn get_indent(&self) -> u32 {
        self.indent
    }

    fn write_direct(&mut self, data: &str) {
        print!("{}", data);
    }
}

pub struct StringPrinter {
    indent: u32,
    data: String,
}

impl Default for StringPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl StringPrinter {
    pub fn new() -> Self {
        StringPrinter {
            indent: 0,
            data: String::new(),
        }
    }

    pub fn get(&self) -> String {
        self.data.clone()
    }
}

impl IRFormatter for StringPrinter {
    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    fn get_indent(&self) -> u32 {
        self.indent
    }

    fn write_direct(&mut self, data: &str) {
        self.data += data;
    }
}
