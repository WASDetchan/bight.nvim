use bight::table::cell::CellPos;

use crate::editor::CELL_UNIT_WIDTH;

pub fn cell_pos((cursorx, cursory): (usize, usize)) -> CellPos {
    ((cursorx - 1) / CELL_UNIT_WIDTH, cursory - 1).into()
}
