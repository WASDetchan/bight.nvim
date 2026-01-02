use std::sync::Arc;

use crate::editor::{EDITORS, render_buffer, util::cell_pos};

use super::CELL_UNIT_WIDTH;
use nvim_oxi::api::{self as nvim, Buffer, opts::SetKeymapOpts, types::Mode};

pub fn add_keymaps(buffer: &mut Buffer) {
    buffer
        .set_keymap(
            Mode::Normal,
            "l",
            &format!("{CELL_UNIT_WIDTH}l"),
            &SetKeymapOpts::builder().noremap(true).build(),
        )
        .unwrap();
    buffer
        .set_keymap(
            Mode::Normal,
            "h",
            &format!("{CELL_UNIT_WIDTH}h"),
            &SetKeymapOpts::builder().noremap(true).build(),
        )
        .unwrap();
    let bufnr = buffer.handle();
    buffer
        .set_keymap(
            Mode::Normal,
            "yy",
            "",
            &SetKeymapOpts::builder()
                .callback(move |()| {
                    let mut editors = EDITORS.lock().unwrap();
                    let editor = editors.get_mut(&bufnr).unwrap();
                    let (row, col) = nvim::get_current_win().get_cursor().unwrap();
                    editor.cursor = cell_pos((col + 1, row));
                    editor.clipboard.set(
                        editor
                            .table
                            .get_source(editor.cursor)
                            .unwrap_or(&Arc::from(""))
                            .clone(),
                    )
                })
                .build(),
        )
        .unwrap();
    let bufnr = buffer.handle();
    let mut buf = buffer.clone();
    buffer
        .set_keymap(
            Mode::Normal,
            "p",
            "",
            &SetKeymapOpts::builder()
                .callback(move |()| {
                    let mut editors = EDITORS.lock().unwrap();
                    let editor = editors.get_mut(&bufnr).unwrap();
                    let (row, col) = nvim::get_current_win().get_cursor().unwrap();
                    editor.cursor = cell_pos((col + 1, row));
                    editor
                        .table
                        .set_source(editor.cursor, editor.clipboard.get());
                    drop(editors);
                    render_buffer(&mut buf);
                })
                .noremap(true)
                .build(),
        )
        .unwrap();
}
