mod clock;
mod print;

use crate::functions::LoxFunction;
use clock::lox_builtin_clock;
use print::lox_builtin_print;

pub fn builtins() -> Vec<LoxFunction> {
    let bltins: Vec<LoxFunction> = vec![
        LoxFunction::new_builtin("clock".to_string(), Some(0), lox_builtin_clock),
        LoxFunction::new_builtin("__builtin_print".to_string(), None, lox_builtin_print),
    ];

    bltins
}
