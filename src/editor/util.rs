use bight::table::cell::CellPos;

use crate::editor::CELL_UNIT_WIDTH;

use nvim_oxi as nvim;

pub fn cell_pos((cursorx, cursory): (usize, usize)) -> CellPos {
    ((cursorx) / CELL_UNIT_WIDTH, cursory - 1).into()
}

pub fn current_cell_pos() -> CellPos {
    let (row, col) = nvim::api::get_current_win().get_cursor().unwrap();
    cell_pos((col, row))
}
