use std::sync::Arc;

use crate::editor::{
    CELL_WIDTH, Editor, autocmd::start_editing_cell, render_buffer, util::current_cell_pos,
};

use super::CELL_UNIT_WIDTH;
use bight::table::slice::SlicePos;
use nvim_oxi::api::{Buffer, opts::SetKeymapOpts, types::Mode};

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
            Mode::VisualSelect,
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
    buffer
        .set_keymap(
            Mode::VisualSelect,
            "l",
            &format!("{CELL_UNIT_WIDTH}l"),
            &SetKeymapOpts::builder().noremap(true).build(),
        )
        .unwrap();
    buffer
        .set_keymap(
            Mode::Normal,
            "v",
            &format!("<C-V>{}l", CELL_WIDTH),
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
                        let pos = current_cell_pos();
                        let mut editor = editor.lock().unwrap();
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
                        let pos = current_cell_pos();
                        start_editing_cell(pos, editor.clone());
                        eprintln!("started editing {pos}");
                    })
                    // .noremap(true)
                    .build(),
            )
            .unwrap();
    }
    {
        let editor = editor.clone();
        buffer
            .set_keymap(
                Mode::Normal,
                "p",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        {
                            let mut editor = editor.lock().unwrap();
                            let pos = current_cell_pos();
                            editor.visual_start = pos;
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
    let cb = {
        let editor = editor.clone();
        move |()| {
            let c_editor = editor.clone();
            let mut editor = editor.lock().unwrap();

            let start = editor.visual_start;
            let mut end = current_cell_pos();
            end.x += 1;
            end.y += 1;
            let slice = SlicePos::from((start, end));

            for row in slice.rows() {
                for col in slice.columns() {
                    let mut pos = slice.start;
                    pos.x += col;
                    pos.y += row;
                    editor.table.set_source::<Arc<str>>(pos, None);
                }
            }
            drop(editor);
            render_buffer(c_editor);
        }
    };
    buffer
        .set_keymap(
            Mode::VisualSelect,
            "d",
            "",
            &SetKeymapOpts::builder().callback(cb).noremap(true).build(),
        )
        .unwrap();
    {
        let cb = move |()| {
            let c_editor = editor.clone();
            let mut editor = editor.lock().unwrap();

            let start = editor.visual_start;
            let mut end = current_cell_pos();
            end.x += 1;
            end.y += 1;
            let slice = SlicePos::from((start, end));

            dbg!(slice);
            let source = editor.clipboard.get();
            for row in slice.rows() {
                for col in slice.columns() {
                    let mut pos = slice.start;
                    pos.x += col;
                    pos.y += row;
                    editor.table.set_source(pos, source.clone());
                }
            }
            drop(editor);
            render_buffer(c_editor);
        };
        buffer
            .set_keymap(
                Mode::VisualSelect,
                "p",
                "",
                &SetKeymapOpts::builder().callback(cb).noremap(true).build(),
            )
            .unwrap();
    }
}
