pub struct Options {
    pub dump_ops: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { 
            dump_ops: false,
        }
    }
}