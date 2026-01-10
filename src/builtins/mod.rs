use std::rc::Rc;

mod clock;
mod print;

use crate::interpreter::LoxCallable;
use clock::ClockBuiltin;
use print::PrintBuiltin;

pub fn builtins() -> Vec<Rc<dyn LoxCallable>> {
    let bltins: Vec<Rc<dyn LoxCallable>> =
        vec![Rc::new(ClockBuiltin::new()), Rc::new(PrintBuiltin::new())];

    bltins
}
