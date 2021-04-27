use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{MalVal, NativeFn};

#[derive(Clone, Debug, PartialEq)]
pub struct Environment(Rc<RefCell<EnvironmentInner>>);

#[derive(Clone, Debug, PartialEq)]
struct EnvironmentInner {
    parent: Option<Environment>,
    builtin: HashMap<String, NativeFn>,
    data: HashMap<String, MalVal>,
}

#[derive(Clone)]
pub enum EnvVal {
    NativeFn(NativeFn),
    Val(MalVal),
}

pub struct EnvironmentBuilder {
    env: EnvironmentInner,
}

impl EnvironmentBuilder {
    pub fn new() -> Self {
        EnvironmentBuilder {
            env: EnvironmentInner {
                parent: None,
                builtin: HashMap::new(),
                data: HashMap::new(),
            },
        }
    }

    pub fn with_parent(mut self, env: &Environment) -> Self {
        self.env.parent = Some(env.clone());
        self
    }

    pub fn with_builtins(mut self, fs: HashMap<String, NativeFn>) -> Self {
        for (sym_name, f) in fs {
            self.env.builtin.insert(sym_name, f);
        }
        self
    }

    pub fn build(self) -> Environment {
        Environment(Rc::new(RefCell::new(self.env)))
    }
}

impl Environment {
    pub fn set(&self, sym_name: String, val: MalVal) {
        self.0.borrow_mut().data.insert(sym_name, val);
    }

    pub fn get(&self, sym_name: &str) -> Option<EnvVal> {
        self.find(sym_name).map(|e| {
            let env = e.0.borrow();
            if env.builtin.contains_key(sym_name) {
                let f = env.builtin[sym_name];
                EnvVal::NativeFn(f)
            } else if env.data.contains_key(sym_name) {
                let v = env.data[sym_name].clone();
                EnvVal::Val(v)
            } else {
                unreachable!()
            }
        })
    }

    pub fn find(&self, sym_name: &str) -> Option<Environment> {
        if self.0.borrow().data.contains_key(sym_name)
            || self.0.borrow().builtin.contains_key(sym_name)
        {
            Some(Environment(self.0.clone()))
        } else if let Some(parent) = &self.0.borrow().parent {
            parent.find(sym_name)
        } else {
            None
        }
    }
}
