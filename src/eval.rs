use crate::types::{MalAtom, MalVal};
use thiserror::Error;

pub struct Environment {}
type MalFunc = fn(Vec<MalVal>) -> Result<MalVal>;

impl Environment {
    pub fn new() -> Self {
        Environment {}
    }

    pub fn is_symbol_defined(&self, sym_name: &str) -> bool {
        matches!(sym_name, "+" | "-" | "*")
    }

    fn lookup_func(&self, sym_name: &str) -> Option<MalFunc> {
        match sym_name {
            "+" => Some(|args: Vec<MalVal>| {
                let mut acc: i64 = 0;
                for v in args.into_iter() {
                    if let MalVal::Atom(MalAtom::Int(num)) = v {
                        acc += num;
                    } else {
                        return Err(EvalError::NotANumber);
                    }
                }
                Ok(MalVal::Atom(MalAtom::Int(acc)))
            }),
            "-" => Some(|args: Vec<MalVal>| {
                let mut first = true;
                let mut acc: i64 = 0;
                let mut count = 0;
                for v in args.into_iter() {
                    if let MalVal::Atom(MalAtom::Int(num)) = v {
                        count += 1;
                        if first {
                            acc = num;
                            first = false;
                        } else {
                            acc -= num;
                        }
                    } else {
                        return Err(EvalError::NotANumber);
                    }
                }
                if count == 1 {
                    Ok(MalVal::Atom(MalAtom::Int(-acc)))
                } else {
                    Ok(MalVal::Atom(MalAtom::Int(acc)))
                }
            }),
            "*" => Some(|args: Vec<MalVal>| {
                let mut acc: i64 = 1;
                for v in args.into_iter() {
                    if let MalVal::Atom(MalAtom::Int(num)) = v {
                        acc *= num;
                    } else {
                        return Err(EvalError::NotANumber);
                    }
                }
                Ok(MalVal::Atom(MalAtom::Int(acc)))
            }),
            _ => None,
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
    #[error("Function {0} not defined")]
    FunctionUndefined(String),
    #[error("Bad function designator {0}")]
    BadFunctionDesignator(String),
}

pub fn eval(ast: MalVal, env: &mut Environment) -> Result<MalVal> {
    match ast {
        MalVal::List(list) => {
            if list.is_empty() {
                Ok(MalVal::List(list))
            } else {
                let evaluated = eval_ast(MalVal::List(list), env)?;

                if let MalVal::List(mut list) = evaluated {
                    // TODO: removing the first element of a vector is not great
                    // as it shuffles all the values left by one
                    let sym = list.remove(0);
                    if let MalVal::Atom(MalAtom::Sym(sym_name)) = sym {
                        if let Some(f) = env.lookup_func(&sym_name) {
                            Ok(f(list)?)
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
        _ => eval_ast(ast, env),
    }
}

fn eval_ast(ast: MalVal, env: &mut Environment) -> Result<MalVal> {
    match ast {
        MalVal::Atom(atom) => match atom {
            MalAtom::Sym(sym) => {
                if env.is_symbol_defined(&sym) {
                    Ok(MalVal::Atom(MalAtom::Sym(sym)))
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
