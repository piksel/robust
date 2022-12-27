pub struct Options {
    pub dump_ops: bool,
    pub history_len: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self { 
            dump_ops: false,
            history_len: 0,
        }
    }
}