use std::sync::{LazyLock, Mutex};

use nvim_oxi::{Dictionary, Function, Object};

static COUNT: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0u64));

fn setup(_: ()) -> u64 {
    *COUNT.lock().unwrap() += 1;
    return *COUNT.lock().unwrap();
}

#[nvim_oxi::plugin]
fn bight() -> Dictionary {
    let add = Function::from_fn(|(a, b): (i32, i32)| a + b);

    let multiply = Function::from_fn(|(a, b): (i32, i32)| a * b);

    let compute = Function::from_fn(|(fun, a, b): (Function<(i32, i32), i32>, i32, i32)| {
        fun.call((a, b)).unwrap()
    });

    Dictionary::from_iter([
        ("add", Object::from(add)),
        ("multiply", Object::from(multiply)),
        ("compute", Object::from(compute)),
        ("setup", Object::from(Function::from_fn(setup))),
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
