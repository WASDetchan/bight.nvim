use std::sync::Arc;

use crate::editor::{CELL_WIDTH, Editor, render_buffer};
use crate::util::{
    self, current_cell_pos, move_left, move_left_visual, move_right, move_right_visual,
};

use bight::table::slice::CellRange;
use nvim_oxi::api::{Buffer, opts::SetKeymapOpts, types::Mode};

pub fn add_keymaps(buffer: &mut Buffer, editor: Editor) {
    buffer
        .set_keymap(
            Mode::Normal,
            "l",
            "",
            &SetKeymapOpts::builder().callback(|()| move_right()).build(),
        )
        .unwrap();
    buffer
        .set_keymap(
            Mode::VisualSelect,
            "l",
            "",
            &SetKeymapOpts::builder()
                .callback(|()| move_right_visual())
                .build(),
        )
        .unwrap();
    buffer
        .set_keymap(
            Mode::Normal,
            "h",
            "",
            &SetKeymapOpts::builder().callback(|()| move_left()).build(),
        )
        .unwrap();
    buffer
        .set_keymap(
            Mode::VisualSelect,
            "h",
            "",
            &SetKeymapOpts::builder()
                .callback(|()| move_left_visual())
                .build(),
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
                    .callback(move |()| editor.yank_current_source())
                    .build(),
            )
            .unwrap();
    }
    {
        let editor = editor.clone();
        buffer
            .set_keymap(
                Mode::Normal,
                "dd",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        editor.yank_current_source();
                        editor.set_source(util::current_cell_pos(), String::from(""));
                        editor.render();
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
                "cc",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| {
                        editor.yank_current_source();
                        editor.set_source(util::current_cell_pos(), String::from(""));
                        editor.render();
                        nvim_oxi::api::command("startinsert").unwrap();
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
                "Y",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| editor.yank_current_value())
                    .build(),
            )
            .unwrap();
    }
    {
        let editor = editor.clone();
        buffer
            .set_keymap(
                Mode::VisualSelect,
                "Y",
                "",
                &SetKeymapOpts::builder()
                    .callback(move |()| editor.yank_current_value_range_as_csv())
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
                        editor.start_editing_cell(pos);
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
                        editor.render();
                    })
                    .noremap(true)
                    .build(),
            )
            .unwrap();
    }
    let cb = {
        let editor = editor.clone();
        move |()| {
            let start = editor.state().visual_start;
            let mut end = current_cell_pos();
            end.x += 1;
            end.y += 1;
            let slice = CellRange::from((start, end));

            for row in slice.rows() {
                for col in slice.columns() {
                    let mut pos = slice.start;
                    pos.x += col;
                    pos.y += row;
                    editor.state().table.set_source::<Arc<str>>(pos, None);
                }
            }
            editor.render();
            render_buffer(&editor);
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
            let start = editor.state().visual_start;
            let mut end = current_cell_pos();
            end.x += 1;
            end.y += 1;
            let slice = CellRange::from((start, end));

            let source = editor.state().clipboard.get();
            for row in slice.rows() {
                for col in slice.columns() {
                    let mut pos = slice.start;
                    pos.x += col;
                    pos.y += row;
                    editor.state().table.set_source(pos, source.clone());
                }
            }
            editor.render();
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
