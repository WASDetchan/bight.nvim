mod api;
mod autocmd;
mod clipboard;
mod key;
pub use autocmd::attach_editor_autocmd;
use hashbrown::HashMap;
pub use key::add_keymaps;

use std::{
    ops::Deref,
    path::Path,
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

use bight::{
    clipboard::Clipboard,
    evaluator::{EvaluatorTable, SourceTable, TableValue},
    file::slice_to_csv_string,
    table::{CellRange, Table, cell::CellPos, slice::row::RowSlice},
};
use nvim_oxi::{
    self as nvim,
    api::{
        Buffer,
        opts::{OptionOpts, SetExtmarkOpts},
    },
};

use crate::util::{self, cursor_position, get_buffer_line};

pub struct EditorState {
    edit: Option<CellPos>,
    visual_start: CellPos,
    buffer: Buffer,
    table: EvaluatorTable,
    clipboard: Clipboard,
}

impl EditorState {
    pub fn with_new_buffer(buffer: Buffer) -> Self {
        Self {
            buffer,
            edit: None,
            visual_start: CellPos::default(),
            table: EvaluatorTable::new(SourceTable::new()),
            clipboard: Clipboard::new(),
        }
    }
    pub fn with_file_buffer(buffer: Buffer, file: &Path) -> anyhow::Result<Self> {
        let source = bight::file::load(file)?.source;
        Ok(Self {
            buffer,
            edit: None,
            visual_start: CellPos::default(),
            table: EvaluatorTable::new(source),
            clipboard: Clipboard::new(),
        })
    }
}

pub const CELL_WIDTH: usize = 8;
pub const CELL_SEPARATOR: &str = " ";
pub const CELL_UNIT_WIDTH: usize = CELL_WIDTH + CELL_SEPARATOR.len();

static EDITORS: LazyLock<Mutex<HashMap<i32, Editor>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct Editor {
    inner: Arc<Mutex<EditorState>>,
}

impl Deref for Editor {
    type Target = Arc<Mutex<EditorState>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Editor {
    pub fn with_new_buffer(buffer: Buffer) -> Self {
        let bufnr = buffer.handle();
        let this = Self {
            inner: Arc::new(Mutex::new(EditorState::with_new_buffer(buffer))),
        };
        EDITORS.lock().unwrap().insert(bufnr, this.clone());
        this
    }
    pub fn with_file_buffer(buffer: Buffer, file: &Path) -> anyhow::Result<Self> {
        let bufnr = buffer.handle();
        let this = Self {
            inner: Arc::new(Mutex::new(EditorState::with_file_buffer(buffer, file)?)),
        };
        EDITORS.lock().unwrap().insert(bufnr, this.clone());
        Ok(this)
    }
    pub fn of_existing_buffer(bufnr: Option<i32>) -> Option<Self> {
        let mut bufnr = bufnr.unwrap_or_default();
        if bufnr == 0 {
            bufnr = nvim::api::get_current_buf().handle();
        }
        EDITORS.lock().unwrap().get(&bufnr).cloned()
    }
    pub fn state(&self) -> MutexGuard<'_, EditorState> {
        self.lock().unwrap()
    }
    pub fn exit_edit(&self) {
        let Some(pos) = self.state().edit.take() else {
            return;
        };

        util::set_cursor_to_cell_pos(pos);

        let (line, col) = cursor_position(pos);

        let replace_start_x = col;
        let row = line - 1;

        let buffer = self.state().buffer.clone();

        let source = get_buffer_line(&buffer, row)
            .chars()
            .skip(replace_start_x)
            .collect::<String>()
            .trim()
            .to_string();

        self.state().table.set_source(pos, Some(Arc::from(source)));
    }
    pub fn set_visual_start(&self, pos: CellPos) {
        self.state().visual_start = pos;
    }

    pub fn get_visual_start(&self) -> CellPos {
        self.state().visual_start
    }

    pub fn render(&self) {
        render_buffer(self);
    }

    pub fn get_value(&self, pos: CellPos) -> String {
        self.state()
            .table
            .get(pos)
            .unwrap_or(&TableValue::Empty)
            .to_string()
    }
    pub fn get_source(&self, pos: CellPos) -> String {
        self.state()
            .table
            .get_source(pos)
            .map(|arc| arc.to_string())
            .unwrap_or_default()
    }
    pub fn get_current_visual_range(&self) -> CellRange {
        let start = self.state().visual_start;
        let end = util::current_cell_pos();
        let mut range = CellRange::new_limits(start, end);
        range.width += 1;
        range.height += 1;
        range
    }
    pub fn get_value_range_as_csv(&self, range: CellRange) -> String {
        slice_to_csv_string(self.state().table.slice(range))
    }
    pub fn set_source(&self, pos: CellPos, src: String) {
        self.state().table.set_source(pos, Some(src));
    }
    pub fn yank_source(&self, pos: CellPos) {
        let mut editor = self.state();
        let source = editor
            .table
            .get_source(pos)
            .unwrap_or(&Arc::from(""))
            .clone();

        editor.clipboard.set(source);
    }
    pub fn yank_current_source(&self) {
        let pos = util::current_cell_pos();
        self.yank_source(pos);
    }
    pub fn yank_value(&self, pos: CellPos) {
        self.state().table.evaluate();
        let value = self.get_value(pos);
        let mut editor = self.state();
        editor.clipboard.set(value.into());
    }
    pub fn yank_current_value(&self) {
        let pos = util::current_cell_pos();
        self.yank_value(pos);
    }
    pub fn yank_value_range_as_csv(&self, range: CellRange) {
        let source = self.get_value_range_as_csv(range);
        self.state().clipboard.set(Arc::from(source));
    }
    pub fn yank_current_value_range_as_csv(&self) {
        let range = self.get_current_visual_range();
        self.yank_value_range_as_csv(range);
    }

    pub fn start_editing_cell(&self, pos: CellPos) -> Buffer {
        let buffer = nvim::api::create_buf(true, false).unwrap();
        self.attach_cell_to_buffer(pos, buffer.clone());
        buffer
    }

    pub fn plot_segments(&self, range: CellRange, path: &Path) -> Result<(), anyhow::Error> {
        Ok(bight::plot::plot_segments_to_file(
            self.state().table.slice(range),
            path,
        )?)
    }
    pub fn plot_linear(
        &self,
        range: CellRange,
        path: &Path,
    ) -> Result<Vec<(f64, f64)>, anyhow::Error> {
        Ok(bight::plot::plot_linear_to_file(
            self.state().table.slice(range),
            path,
        )?)
    }

    pub fn attach_cell_to_buffer(&self, pos: CellPos, mut buffer: Buffer) {
        let source = self
            .state()
            .table
            .get_source(pos)
            .map(|s| s.lines().map(String::from).collect::<Vec<_>>())
            .unwrap_or_default();

        buffer.set_lines(.., false, source).unwrap();
        prepare_cell_buffer(pos, &mut buffer);
        autocmd::attach_cell_edit_autocmd(pos, buffer, self.clone());
    }
}

fn prepare_cell_buffer(pos: CellPos, buffer: &mut Buffer) {
    buffer
        .set_name(format!(
            "{pos}-{}",
            std::time::SystemTime::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_millis()
        ))
        .unwrap();
    nvim::api::set_option_value(
        "buftype",
        "acwrite",
        &OptionOpts::builder().buf(buffer.clone()).build(),
    )
    .unwrap();
    nvim::api::set_option_value(
        "filetype",
        "bcell",
        &OptionOpts::builder().buf(buffer.clone()).build(),
    )
    .unwrap();

    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buf(buffer.clone()).build(),
    )
    .unwrap();
}

fn format_row(row: RowSlice<'_, EvaluatorTable>) -> impl Iterator<Item = char> {
    row.into_iter().flat_map(|v| {
        v.unwrap_or(&TableValue::Empty)
            .format_to_length(CELL_WIDTH)
            .chars()
            .collect::<Vec<_>>()
            .into_iter()
            .chain(CELL_SEPARATOR.chars())
    })
}
pub fn render_buffer_edit(editor: &Editor, pos: CellPos, replace_input: bool) {
    let display_width = nvim::api::get_current_win().get_width().unwrap() as usize;
    let height = nvim::api::get_current_win().get_height().unwrap() as usize;

    let width_cells = display_width.div_ceil(CELL_UNIT_WIDTH);

    let (line, col) = cursor_position(pos);

    let replace_start_x = col;
    let replace_y = line - 1;
    let replace_end_x = replace_start_x + display_width;

    let line_to_edit = get_buffer_line(&editor.state().buffer, replace_y);

    let mut editor = editor.lock().unwrap();
    editor.table.evaluate();

    let source = String::from(
        editor
            .table
            .get_source(pos)
            .map_or("", |v| v.lines().next().unwrap_or("")),
    );

    let slice = editor
        .table
        .slice((0, 0)..=(width_cells as isize, height as isize));

    let lines: Vec<_> = slice
        .rows()
        .map(|row| {
            let line = format_row(row);
            if row.into_inner().start().y != replace_y as isize {
                line.take(display_width).collect::<String>()
            } else if replace_input {
                line.take(replace_start_x)
                    .chain(source.chars())
                    .chain(" ".chars().cycle())
                    .take(replace_end_x + 1)
                    .collect()
            } else {
                line.take(replace_start_x)
                    .chain(line_to_edit.chars().skip(replace_start_x))
                    .chain(" ".chars().cycle())
                    .take(replace_end_x + 1)
                    .collect()
            }
        })
        .collect();

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
}

fn render_buffer(editor: &Editor) {
    let replace_pos = editor.lock().unwrap().edit;
    if let Some(pos) = replace_pos {
        render_buffer_edit(editor, pos, false);
        return;
    }
    let display_width = nvim::api::get_current_win().get_width().unwrap() as usize;
    let height = nvim::api::get_current_win().get_height().unwrap() as usize;

    let mut editor = editor.lock().unwrap();
    editor.table.evaluate();

    let width_cells = display_width.div_ceil(CELL_UNIT_WIDTH);

    let slice = editor
        .table
        .slice((0, 0)..=(width_cells as isize, height as isize));

    let lines: Vec<_> = slice
        .rows()
        .map(|row| format_row(row).collect::<String>())
        .collect();

    let mut buffer = editor.buffer.clone();

    drop(editor);

    buffer.set_lines(0..height, false, lines).unwrap();
}
