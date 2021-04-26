use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::{MalAtom, MalVal};
use itertools::Itertools;
use thiserror::Error;

mod builtin;

#[derive(Clone)]
pub struct Environment(Rc<RefCell<EnvironmentInner>>);
pub type MalFunc = fn(Vec<MalVal>) -> Result<MalVal>;

struct EnvironmentInner {
    parent: Option<Environment>,
    builtin: HashMap<String, MalFunc>,
    data: HashMap<String, MalVal>,
}

#[derive(Clone)]
enum EnvVal {
    Func(MalFunc),
    Val(MalVal),
}

impl Environment {
    pub fn new() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentInner {
            parent: None,
            builtin: builtin::defaults(),
            data: HashMap::new(),
        })))
    }

    pub fn new_from(parent: &Environment) -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentInner {
            parent: Some(parent.clone()),
            builtin: builtin::defaults(),
            data: HashMap::new(),
        })))
    }

    fn set(&self, sym_name: String, val: MalVal) {
        self.0.borrow_mut().data.insert(sym_name, val);
    }

    fn get(&self, sym_name: &str) -> Option<EnvVal> {
        self.find(sym_name).map(|e| {
            let env = e.0.borrow();
            if env.builtin.contains_key(sym_name) {
                let f = env.builtin[sym_name];
                EnvVal::Func(f)
            } else if env.data.contains_key(sym_name) {
                let v = env.data[sym_name].clone();
                EnvVal::Val(v)
            } else {
                unreachable!()
            }
        })
    }

    fn find(&self, sym_name: &str) -> Option<Environment> {
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

pub type Result<T> = std::result::Result<T, EvalError>;

#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    #[error("Symbol {0} not in environment")]
    SymbolNotFound(String),
    #[error("Not a number")]
    NotANumber,
    #[error("Not a symbol")]
    NotASymbol,
    #[error("Not a list")]
    NotAList,
    #[error("Function {0} not defined")]
    FunctionUndefined(String),
    #[error("Bad function designator {0}")]
    BadFunctionDesignator(String),
}

#[allow(clippy::clippy::collapsible_else_if)]
pub fn eval(ast: MalVal, env: &Environment) -> Result<MalVal> {
    match ast {
        MalVal::List(mut list) => {
            if list.is_empty() {
                Ok(MalVal::List(list))
            } else {
                if list[0] == MalVal::Atom(MalAtom::Sym("def!".to_owned())) {
                    list.remove(0);
                    let atom = list.remove(0);
                    if let MalVal::Atom(MalAtom::Sym(sym_name)) = atom {
                        let evaluated = eval(list.remove(0), env)?;
                        env.set(sym_name, evaluated.clone());
                        Ok(evaluated)
                    } else {
                        Err(EvalError::NotASymbol)
                    }
                } else if list[0] == MalVal::Atom(MalAtom::Sym("let*".to_owned())) {
                    list.remove(0);
                    if let MalVal::List(vars) = list.remove(0) {
                        let child_env = Environment::new_from(env);
                        let mut it = vars.into_iter();
                        while let Some((sym, to_eval)) = it.next_tuple() {
                            match sym {
                                MalVal::Atom(MalAtom::Sym(sym_name)) => {
                                    let evaluated = eval(to_eval, &child_env)?;
                                    child_env.set(sym_name, evaluated);
                                }
                                _ => return Err(EvalError::NotASymbol),
                            }
                        }
                        let to_eval = list.remove(0);
                        eval(to_eval, &child_env)
                    } else {
                        Err(EvalError::NotAList)
                    }
                } else {
                    let evaluated = eval_ast(MalVal::List(list), env)?;

                    if let MalVal::List(mut list) = evaluated {
                        // TODO: removing the first element of a vector is not great
                        // as it shuffles all the values left by one
                        let sym = list.remove(0);
                        if let MalVal::Atom(MalAtom::Sym(sym_name)) = sym {
                            if let Some(env_val) = env.get(&sym_name) {
                                if let EnvVal::Func(f) = env_val {
                                    Ok(f(list)?)
                                } else {
                                    Err(EvalError::BadFunctionDesignator(sym_name))
                                }
                            } else {
                                Err(EvalError::FunctionUndefined(sym_name))
                            }
                        } else {
                            Err(EvalError::BadFunctionDesignator(sym.to_string()))
                        }
                    } else {
                        panic!("list evaluated to non list")
                    }
                }
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_ast(ast: MalVal, env: &Environment) -> Result<MalVal> {
    match ast {
        MalVal::Atom(atom) => match atom {
            MalAtom::Sym(sym) => {
                if let Some(env_val) = env.get(&sym) {
                    match env_val {
                        EnvVal::Func(_) => Ok(MalVal::Atom(MalAtom::Sym(sym))),
                        EnvVal::Val(v) => Ok(v),
                    }
                } else {
                    Err(EvalError::SymbolNotFound(sym))
                }
            }
            _ => Ok(MalVal::Atom(atom)),
        },
        MalVal::List(list) => {
            let mut evaluated = Vec::new();
            for v in list.into_iter() {
                evaluated.push(eval(v, env)?);
            }
            Ok(MalVal::List(evaluated))
        }
        MalVal::Vector(_) => {
            unimplemented!()
        }
        MalVal::AssocArray(_) => {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        {
            let mut env = Environment::new();
            let ast = MalVal::Atom(MalAtom::Sym("undefined_sym".into()));
            let evaluated = eval(ast, &mut env).unwrap_err();

            assert_eq!(evaluated, EvalError::SymbolNotFound("undefined_sym".into()));
        }
        {
            for op in &["+", "-", "*"] {
                let mut env = Environment::new();
                let ast = MalVal::Atom(MalAtom::Sym((*op).to_owned()));
                let expected = ast.clone();
                let evaluated = eval(ast, &mut env).unwrap();

                assert_eq!(evaluated, expected);
            }
        }
        {
            for atom in vec![
                MalVal::Atom(MalAtom::Nil),
                MalVal::Atom(MalAtom::True),
                MalVal::Atom(MalAtom::False),
                MalVal::Atom(MalAtom::Str("asdf".into())),
                MalVal::Atom(MalAtom::Int(1)),
            ]
            .into_iter()
            {
                let mut env = Environment::new();
                let ast = MalVal::List(vec![atom, MalVal::Atom(MalAtom::Int(2))]);
                let evaluated = eval(ast, &mut env).unwrap_err();
                assert!(matches!(evaluated, EvalError::BadFunctionDesignator(_)))
            }
        }
        {
            for op in &["+", "-", "*"] {
                for atom in vec![
                    MalVal::Atom(MalAtom::Nil),
                    MalVal::Atom(MalAtom::True),
                    MalVal::Atom(MalAtom::False),
                    MalVal::Atom(MalAtom::Str("asdf".into())),
                ]
                .into_iter()
                {
                    let mut env = Environment::new();
                    let ast = MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym((*op).to_owned())),
                        atom,
                        MalVal::Atom(MalAtom::Int(2)),
                    ]);
                    let evaluated = eval(ast, &mut env).unwrap_err();
                    assert!(matches!(evaluated, EvalError::NotANumber))
                }
            }
        }
        {
            for tc in &[("+", 2i64), ("-", -2i64), ("*", 2i64)] {
                let mut env = Environment::new();
                let (op, expected) = *tc;
                let ast = MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym(op.to_owned())),
                    MalVal::Atom(MalAtom::Int(2)),
                ]);
                let evaluated = eval(ast, &mut env).unwrap();

                assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(expected)));
            }
        }
        {
            for tc in &[("+", 9i64), ("-", -5i64), ("*", 24i64)] {
                let mut env = Environment::new();
                let (op, expected) = *tc;
                let ast = MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym(op.to_owned())),
                    MalVal::Atom(MalAtom::Int(2)),
                    MalVal::Atom(MalAtom::Int(3)),
                    MalVal::Atom(MalAtom::Int(4)),
                ]);
                let evaluated = eval(ast, &mut env).unwrap();

                assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(expected)));
            }
        }
    }
}
