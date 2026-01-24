mod api;

pub use api::make as make_api;

use bight::table::cell::CellPos;

use crate::editor::{CELL_UNIT_WIDTH, CELL_WIDTH};

use nvim_oxi::{
    self as nvim,
    api::{Buffer, opts::EchoOpts},
};

pub fn get_cursor() -> (usize, usize) {
    let (row, _) = nvim::api::get_current_win().get_cursor().unwrap();

    let col = lua::nvim_mlua()
        .load("vim.fn.virtcol('.')")
        .eval::<usize>()
        .unwrap()
        .saturating_sub(1);

    (row, col)
}

pub fn set_cursor(line: usize, col: usize) {
    let col = if col > 0 {
        lua::nvim_mlua()
            .load(format!("vim.fn.virtcol2col(0, {line}, {col})"))
            .eval::<usize>()
            .unwrap()
    } else {
        0
    };

    nvim::api::get_current_win().set_cursor(line, col).unwrap();
}

pub fn cell_pos((cursorx, cursory): (usize, usize)) -> CellPos {
    (((cursorx) / CELL_UNIT_WIDTH) as isize, cursory as isize - 1).into()
}
pub fn current_cell_pos() -> CellPos {
    let (row, col) = get_cursor();
    cell_pos((col, row))
}

pub fn cursor_position(pos: CellPos) -> (usize, usize) {
    let x: usize = pos.x.try_into().unwrap_or_default();
    let y: usize = pos.y.try_into().unwrap_or_default();
    (y + 1, x * CELL_UNIT_WIDTH)
}

pub fn normalize_cursor_position(line: usize, col: usize) -> (usize, usize) {
    (line, col - col % CELL_UNIT_WIDTH)
}

pub fn set_cursor_to_cell_pos(pos: CellPos) {
    let (line, col) = cursor_position(pos);

    set_cursor(line, col);
}
pub fn set_cursor_to_cell_pos_visual(pos: CellPos) {
    let (line, mut col) = cursor_position(pos);
    col += CELL_WIDTH;

    set_cursor(line, col);
}

pub fn normalize_cursor() {
    let (line, col) = get_cursor();
    let (line, col) = normalize_cursor_position(line, col);

    set_cursor(line, col);
}

pub fn move_cells(x: isize, y: isize) {
    let mut pos = current_cell_pos();
    pos.x += x;
    pos.y += y;

    if pos.x.is_negative() {
        pos.x = 0;
    }
    if pos.y.is_negative() {
        pos.y = 0;
    }

    set_cursor_to_cell_pos(pos);
}

pub fn move_left() {
    move_cells(-1, 0);
}
pub fn move_right() {
    move_cells(1, 0);
}
pub fn move_up() {
    move_cells(0, -1);
}
pub fn move_down() {
    move_cells(0, 1);
}
pub fn move_cells_visual(x: isize, y: isize) {
    let mut pos = current_cell_pos();
    pos.x += x;
    pos.y += y;

    if pos.x.is_negative() {
        pos.x = 0;
    }
    if pos.y.is_negative() {
        pos.y = 0;
    }

    set_cursor_to_cell_pos_visual(pos);
}

pub fn move_left_visual() {
    move_cells_visual(-1, 0);
}
pub fn move_right_visual() {
    move_cells_visual(1, 0);
}
pub fn move_up_visual() {
    move_cells_visual(0, -1);
}
pub fn move_down_visual() {
    move_cells_visual(0, 1);
}

pub fn notify_err(msg: &str) {
    nvim::api::echo(
        [(msg, Option::<char>::None)],
        true,
        &EchoOpts::builder().err(true).build(),
    )
    .unwrap();
}

pub fn notify(msg: &str) {
    nvim::api::echo(
        [(msg, Option::<char>::None)],
        true,
        &EchoOpts::builder().err(false).build(),
    )
    .unwrap();
}

#[macro_export]
macro_rules! notify {
    ($args:tt) => {
        $crate::util::notify(&format!($args))
    };
}

#[macro_export]
macro_rules! enotify {
    ($args:tt) => {
        $crate::util::notify_err(&format!($args))
    };
}

pub fn get_buffer_line(buffer: &Buffer, line: usize) -> String {
    buffer
        .get_lines(line..=line, false)
        .unwrap()
        .next()
        .unwrap()
        .to_string()
}

pub fn get_buffer_as_string(buffer: &Buffer) -> String {
    buffer
        .get_lines(.., false)
        .unwrap()
        .fold(String::new(), |v, a| format!("{v}{a}\n"))
}

pub use lua::*;

mod lua {
    use nvim_oxi::mlua;
    use std::any::type_name;

    use nvim_oxi::{
        self as nvim, Dictionary, Function, Object, ObjectKind,
        lua::{Error as LuaError, Poppable, Pushable},
    };

    pub fn fn_object<T: Poppable, R: Pushable, F: Fn(T) -> R + 'static>(f: F) -> Object {
        Object::from(Function::from_fn(f))
    }
    pub fn unit_fn_object<R: Pushable, F: Fn() -> R + 'static>(f: F) -> Object {
        Object::from(Function::from_fn(move |()| f()))
    }
    pub fn get_as_bool(dict: &Dictionary, key: &str) -> bool {
        let Some(x) = dict.get(key) else {
            return false;
        };
        match x.kind() {
            ObjectKind::Nil => false,
            ObjectKind::Boolean => unsafe { x.as_boolean_unchecked() },
            _ => true,
        }
    }

    pub fn pop_error<T>(msg: impl ToString) -> nvim::lua::Error {
        LuaError::pop_error(type_name::<T>(), msg.to_string())
    }

    pub fn nvim_mlua() -> mlua::Lua {
        unsafe {
            nvim::lua::with_state(|state| mlua::Lua::get_or_init_from_ptr(state as *mut _).clone())
        }
    }
}
