pub struct Options {
    pub dump_ops: bool,
    pub history_len: usize,


    pub sprite_order_overlay: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { 
            dump_ops: false,
            history_len: 0,
            sprite_order_overlay: false,
        }
    }
}