mod editor;

use nvim_oxi::{
    self as nvim, Dictionary, Function, Object, ObjectKind,
    lua::{Poppable, Pushable},
};

use crate::editor::attach_editor_autocmd;

fn create_filetype() {
    let lua = nvim::mlua::lua();
    let chunk = lua.load(
        r#"vim.filetype.add({
          extension = {
            bight = "bight",
          },
        })"#,
    );
    chunk.exec().unwrap();
}

fn to_bool(x: &Object) -> bool {
    match x.kind() {
        ObjectKind::Nil => false,
        ObjectKind::Boolean => unsafe { x.as_boolean_unchecked() },
        _ => true,
    }
}

fn setup((opts,): (Option<Dictionary>,)) {
    let opts = opts.unwrap_or(Dictionary::new());
    create_filetype();
    if to_bool(opts.get("default_keys").unwrap_or(&Object::nil())) {
        todo!();
    }
    attach_editor_autocmd();
}

fn fn_object<T: Poppable, R: Pushable, F: Fn((T,)) -> R + 'static>(f: F) -> Object {
    Object::from(Function::from_fn(f))
}

#[nvim::plugin]
fn bight() -> Dictionary {
    Dictionary::from_iter([("setup", fn_object(setup))])
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
