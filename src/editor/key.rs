use std::sync::Arc;

use crate::editor::{Editor, autocmd::start_editing_cell, render_buffer, util::cell_pos};

use super::CELL_UNIT_WIDTH;
use nvim_oxi::api::{self as nvim, Buffer, opts::SetKeymapOpts, types::Mode};

pub fn add_keymaps(buffer: &mut Buffer, editor: Editor) {
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
    {
        let editor = editor.clone();
        buffer
            .set_keymap(
                Mode::Normal,
                "yy",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        let (row, col) = nvim::get_current_win().get_cursor().unwrap();
                        let mut editor = editor.lock().unwrap();
                        let pos = cell_pos((col + 1, row));
                        let source = editor
                            .table
                            .get_source(pos)
                            .unwrap_or(&Arc::from(""))
                            .clone();

                        editor.clipboard.set(source)
                    })
                    .build(),
            )
            .unwrap();
    }
    {
        let editor = editor.clone();
        buffer
            .set_keymap(
                Mode::Normal,
                "I",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        let (row, col) = nvim::get_current_win().get_cursor().unwrap();
                        let pos = cell_pos((col + 1, row));
                        start_editing_cell(pos, editor.clone());
                        eprintln!("started editing {pos}");
                    })
                    // .noremap(true)
                    .build(),
            )
            .unwrap();
    }
    {
        buffer
            .set_keymap(
                Mode::Normal,
                "p",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        {
                            let mut editor = editor.lock().unwrap();
                            let (row, col) = nvim::get_current_win().get_cursor().unwrap();
                            let pos = cell_pos((col + 1, row));
                            let source = editor.clipboard.get();
                            editor.table.set_source(pos, source);
                        }
                        render_buffer(editor.clone());
                    })
                    .noremap(true)
                    .build(),
            )
            .unwrap();
    }
}
