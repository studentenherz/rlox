use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::LoxError;
use crate::instances::LoxInstance;
use crate::values::Value;

#[derive(Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Value>,
    superclass: Option<Rc<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Value>,
    ) -> Self {
        Self {
            name: name.to_string(),
            methods,
            superclass,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn call(
        &self,
        ctx: &mut crate::interpreter::InterpreterCtx,
        arguments: &[crate::values::Value],
    ) -> Result<crate::values::Value, LoxError> {
        let instance = Value::instance(LoxInstance::new(self.clone()));

        if let Some(initializer) = self.get_method("init") {
            let init = unsafe { initializer.try_get_function().unwrap_unchecked() };
            init.bind(instance.clone()).call(ctx, arguments)?;
        }

        Ok(instance)
    }

    pub fn arity(&self) -> Option<usize> {
        self.get_method("init")
            .map(|f| unsafe { f.try_get_function().unwrap_unchecked().arity() })
            .unwrap_or(Some(0))
    }

    pub fn get_method(&self, name: &str) -> Option<Value> {
        self.methods
            .get(name)
            .cloned()
            .or(if let Some(cls) = &self.superclass {
                cls.get_method(name)
            } else {
                None
            })
    }
}
