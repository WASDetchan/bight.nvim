mod autocmd;
mod clipboard;
mod key;
mod util;
pub use autocmd::attach_editor_autocmd;
pub use key::add_keymaps;

use std::sync::{LazyLock, Mutex};

use bight::{editor::EditorState, table::Table};
use hashbrown::HashMap;
use nvim_oxi::{
    self as nvim,
    api::{Buffer, opts::OptionOpts},
};

static EDITORS: LazyLock<Mutex<HashMap<i32, EditorState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const CELL_WIDTH: usize = 8;
const CELL_SEPARATOR: &str = " ";
const CELL_UNIT_WIDTH: usize = CELL_WIDTH + CELL_SEPARATOR.len();

fn render_buffer(buffer: &mut Buffer) {
    let mut editors_guard = EDITORS.lock().unwrap();
    let Some(editor) = editors_guard.get_mut(&buffer.handle()) else {
        return;
    };

    editor.table.evaluate();

    let display_width = nvim::api::get_current_win().get_width().unwrap() as usize;
    let height = nvim::api::get_current_win().get_height().unwrap() as usize;

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
        lines.push(line.take(display_width).collect::<String>());
    }

    buffer.set_lines(0..height, false, lines).unwrap();
    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
}
