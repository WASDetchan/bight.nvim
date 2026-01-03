mod autocmd;
mod clipboard;
mod key;
mod util;
pub use autocmd::attach_editor_autocmd;
pub use key::add_keymaps;

use std::sync::{Arc, Mutex};

use bight::{
    clipboard::Clipboard,
    evaluator::EvaluatorTable,
    table::{Table, cell::CellPos},
};
use nvim_oxi::{
    self as nvim,
    api::{
        Buffer,
        opts::{OptionOpts, SetExtmarkOpts},
    },
};

pub struct EditorState {
    replace_mode: Option<CellPos>,
    visual_start: CellPos,
    buffer: Buffer,
    table: EvaluatorTable,
    clipboard: Clipboard,
}

const CELL_WIDTH: usize = 8;
const CELL_SEPARATOR: &str = " ";
const CELL_UNIT_WIDTH: usize = CELL_WIDTH + CELL_SEPARATOR.len();

type Editor = Arc<Mutex<EditorState>>;

pub fn exit_insert(editor: &Editor) {
    let Some(pos) = editor.lock().unwrap().replace_mode.take() else {
        return;
    };

    nvim::api::get_current_win()
        .set_cursor(pos.y + 1, pos.x * CELL_UNIT_WIDTH)
        .unwrap();

    let replace_start_x = pos.x * CELL_UNIT_WIDTH;
    let row = pos.y;

    let source = editor
        .lock()
        .unwrap()
        .buffer
        .get_lines(row..=row, false)
        .unwrap()
        .next()
        .unwrap()
        .to_string()
        .chars()
        .skip(replace_start_x)
        .collect::<String>()
        .trim()
        .to_string();
    editor
        .lock()
        .unwrap()
        .table
        .set_source(pos, Some(Arc::from(source)));
}

pub fn render_buffer_replace(editor: Editor, pos: CellPos, replace_input: bool) {
    let display_width = nvim::api::get_current_win().get_width().unwrap() as usize;
    let height = nvim::api::get_current_win().get_height().unwrap() as usize;

    let replace_start_x = pos.x * CELL_UNIT_WIDTH;
    let replace_y = pos.y;
    let replace_end_x = replace_start_x + display_width;

    let mut editor = editor.lock().unwrap();
    editor.table.evaluate();

    let width_cells = display_width.div_ceil(CELL_UNIT_WIDTH);

    let mut lines = Vec::with_capacity(height);

    for row in 0..height {
        let mut line: Box<dyn Iterator<Item = char>> = Box::new("".chars());
        for col in 0..width_cells {
            let value = editor.table.get((col, row).into());
            let s = value
                .map(|v| v.format_to_length(CELL_WIDTH))
                .unwrap_or(" ".repeat(CELL_WIDTH));
            line = Box::new(
                line.chain(
                    s.chars()
                        .collect::<Vec<_>>() // Because into_chars() is unstable
                        .into_iter()
                        .chain(CELL_SEPARATOR.chars()),
                ),
            );
        }
        let s = if row != replace_y {
            line.take(display_width).collect::<String>()
        } else if replace_input {
            editor.table.evaluate();
            line.take(replace_start_x)
                .chain(
                    editor
                        .table
                        .get_source(pos)
                        .map_or("", |v| v.lines().next().unwrap_or(""))
                        .chars(),
                )
                .chain(" ".chars().cycle())
                .take(replace_end_x + 1)
                .collect()
        } else {
            let buffer_line = editor
                .buffer
                .get_lines(row..=row, false)
                .unwrap()
                .next()
                .unwrap()
                .to_string();

            line.take(replace_start_x)
                .chain(buffer_line.chars().skip(replace_start_x))
                .chain(" ".chars().cycle())
                .take(replace_end_x + 1)
                .collect()
        };
        assert!(!s.contains('\n'));

        lines.push(s);
    }

    let mut buffer = editor.buffer.clone();
    drop(editor);

    buffer.set_lines(0..height, false, lines).unwrap();

    const REPLACE_MARK_ID: u32 = 2724982938; // some random number

    let namespace = nvim::api::create_namespace("BightReplaceMark");

    buffer
        .set_extmark(
            namespace,
            replace_y,
            replace_start_x,
            &SetExtmarkOpts::builder()
                .id(REPLACE_MARK_ID)
                .end_col(replace_end_x)
                .hl_group("Visual")
                .build(),
        )
        .unwrap();

    nvim::api::set_option_value(
        "modified",
        true,
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
}

fn render_buffer(editor: Editor) {
    let replace_pos = editor.lock().unwrap().replace_mode;
    if let Some(pos) = replace_pos {
        render_buffer_replace(editor, pos, false);
        return;
    }
    let display_width = nvim::api::get_current_win().get_width().unwrap() as usize;
    let height = nvim::api::get_current_win().get_height().unwrap() as usize;

    let mut editor = editor.lock().unwrap();
    editor.table.evaluate();

    let table = &editor.table;

    let width_cells = display_width.div_ceil(CELL_UNIT_WIDTH);

    let mut lines = Vec::with_capacity(height);

    for row in 0..height {
        let mut line: Box<dyn Iterator<Item = char>> = Box::new("".chars());
        for col in 0..width_cells {
            let value = table.get((col, row).into());
            let s = value
                .map(|v| v.format_to_length(CELL_WIDTH))
                .unwrap_or(" ".repeat(CELL_WIDTH));
            line = Box::new(
                line.chain(
                    s.chars()
                        .collect::<Vec<_>>() // Because into_chars() is unstable
                        .into_iter()
                        .chain(CELL_SEPARATOR.chars()),
                ),
            );
        }
        let s = line.take(display_width).collect::<String>();
        assert!(!s.contains('\n'));

        lines.push(s);
    }

    let mut buffer = editor.buffer.clone();
    drop(editor);

    buffer.set_lines(0..height, false, lines).unwrap();
    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
}
