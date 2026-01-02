mod autocmd;
mod clipboard;
mod key;
mod util;
pub use autocmd::attach_editor_autocmd;
pub use key::add_keymaps;

use std::sync::{Arc, Mutex};

use bight::{clipboard::Clipboard, evaluator::EvaluatorTable, table::Table};
use nvim_oxi::{
    self as nvim,
    api::{Buffer, opts::OptionOpts},
};

pub struct EditorState {
    buffer: Buffer,
    table: EvaluatorTable,
    clipboard: Clipboard,
}

const CELL_WIDTH: usize = 8;
const CELL_SEPARATOR: &str = " ";
const CELL_UNIT_WIDTH: usize = CELL_WIDTH + CELL_SEPARATOR.len();

type Editor = Arc<Mutex<EditorState>>;

fn render_buffer(editor: Editor) {
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
