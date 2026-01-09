pub mod editor;
pub mod util;

use nvim_oxi::{self as nvim, Dictionary};

use crate::{
    editor::{Editor, attach_editor_autocmd},
    util::{fn_object, get_as_bool},
};

fn create_filetype() {
    let lua = util::nvim_mlua();
    let chunk = lua.load(
        r#"vim.filetype.add({
          extension = {
            bight = "bight",
          },
        })"#,
    );
    chunk.exec().unwrap();
}

fn setup(opts: Option<Dictionary>) {
    let opts = opts.unwrap_or_default();
    create_filetype();
    if get_as_bool(&opts, "default_keys") {
        todo!();
    }
    attach_editor_autocmd();
}

#[nvim::plugin]
fn bight() -> Dictionary {
    let util = util::make_api();
    Dictionary::from_iter([
        ("setup", fn_object(setup)),
        ("buffer_editor", fn_object(Editor::of_existing_buffer)),
        ("util", util.into()),
    ])
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod test {
    use super::*;
    #[nvim_oxi::test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
