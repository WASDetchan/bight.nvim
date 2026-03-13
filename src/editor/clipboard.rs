use bight::clipboard::ClipboardProvider;
use nvim_oxi::mlua;

use crate::util;

pub struct NvimClipboard;

impl ClipboardProvider for NvimClipboard {
    fn get_str(&mut self) -> Option<String> {
        util::nvim_mlua().load("vim.fn.getreg()").eval().ok()
    }
    fn set_str(&mut self, v: &str) {
        util::nvim_mlua()
            .load("function(s) vim.fn.setreg(vim.v.register, s) end")
            .eval::<mlua::Function>()
            .unwrap()
            .call(v)
            .unwrap()
    }
}
