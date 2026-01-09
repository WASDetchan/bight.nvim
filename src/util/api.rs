use super::*;
use nvim_oxi::Dictionary;

pub fn make() -> Dictionary {
    let mut api = Dictionary::new();
    api.insert(
        "get_cell_pos",
        fn_object(|()| {
            let pos = current_cell_pos();
            (pos.x, pos.y)
        }),
    );
    api.insert(
        "set_cursor_to_cell_pos",
        fn_object(|(x, y)| {
            let pos = CellPos::from((x, y));
            set_cursor_to_cell_pos(pos);
        }),
    );
    api.insert(
        "normalize_cursor",
        fn_object(|()| {
            normalize_cursor();
        }),
    );
    api.insert(
        "move_cells",
        fn_object(|(x, y)| {
            move_cells(x, y);
        }),
    );
    api.insert("move_left", unit_fn_object(move_left));
    api.insert("move_right", unit_fn_object(move_right));
    api.insert("move_up", unit_fn_object(move_up));
    api.insert("move_down", unit_fn_object(move_down));

    api
}
