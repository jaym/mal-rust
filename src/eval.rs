use crate::types::{
    env::{EnvVal, Environment, EnvironmentBuilder},
    EvalError, EvalResult, MalAtom, MalFn, MalVal,
};
use itertools::Itertools;

pub mod builtin;

pub fn eval(ast: MalVal, env: &Environment) -> EvalResult<MalVal> {
    match ast {
        MalVal::List(list) => {
            if list.is_empty() {
                Ok(MalVal::List(list))
            } else if list[0] == MalVal::Atom(MalAtom::Sym("def!".to_owned())) {
                handle_def(env, list)
            } else if list[0] == MalVal::Atom(MalAtom::Sym("let*".to_owned())) {
                handle_let(env, list)
            } else if list[0] == MalVal::Atom(MalAtom::Sym("fn*".to_owned())) {
                handle_fn(env, list)
            } else if list[0] == MalVal::Atom(MalAtom::Sym("do".to_owned())) {
                handle_do(env, list)
            } else {
                let evaluated = eval_ast(MalVal::List(list), env)?;

                if let MalVal::List(mut list) = evaluated {
                    // TODO: removing the first element of a vector is not great
                    // as it shuffles all the values left by one
                    let sym = list.remove(0);
                    if let MalVal::Atom(MalAtom::Sym(sym_name)) = sym {
                        if let Some(env_val) = env.get(&sym_name) {
                            if let EnvVal::NativeFn(f) = env_val {
                                Ok(f(list)?)
                            } else {
                                Err(EvalError::BadFunctionDesignator(sym_name))
                            }
                        } else {
                            Err(EvalError::FunctionUndefined(sym_name))
                        }
                    } else if let MalVal::Fn(fbox) = sym {
                        let f = *fbox;
                        if f.binds.len() != list.len() {
                            return Err(EvalError::InvalidArgs);
                        }
                        let child_env = EnvironmentBuilder::new().with_parent(&f.env).build();
                        for (s, v) in f.binds.into_iter().zip(list.into_iter()) {
                            child_env.set(s, v);
                        }
                        eval(f.body, &child_env)
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

fn eval_ast(ast: MalVal, env: &Environment) -> EvalResult<MalVal> {
    match ast {
        MalVal::Atom(atom) => match atom {
            MalAtom::Sym(sym) => {
                if let Some(env_val) = env.get(&sym) {
                    match env_val {
                        EnvVal::NativeFn(_) => Ok(MalVal::Atom(MalAtom::Sym(sym))),
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
        MalVal::Fn(_) => {
            unreachable!()
        }
    }
}

fn handle_def(env: &Environment, mut list: Vec<MalVal>) -> EvalResult<MalVal> {
    if list.len() != 3 {
        return Err(EvalError::InvalidArgs);
    }
    list.remove(0);
    let atom = list.remove(0);
    if let MalVal::Atom(MalAtom::Sym(sym_name)) = atom {
        let evaluated = eval(list.remove(0), env)?;
        env.set(sym_name, evaluated.clone());
        Ok(evaluated)
    } else {
        Err(EvalError::NotASymbol)
    }
}
fn handle_let(env: &Environment, mut list: Vec<MalVal>) -> EvalResult<MalVal> {
    if list.len() != 3 {
        return Err(EvalError::InvalidArgs);
    }
    list.remove(0);
    if let MalVal::List(vars) = list.remove(0) {
        let child_env = EnvironmentBuilder::new().with_parent(env).build();
        if vars.len() % 2 != 0 {
            return Err(EvalError::InvalidArgs);
        }
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
}

fn handle_fn(env: &Environment, mut list: Vec<MalVal>) -> EvalResult<MalVal> {
    if list.len() != 3 {
        return Err(EvalError::InvalidArgs);
    }
    list.remove(0);
    if let MalVal::List(vars) = list.remove(0) {
        let mut binds = Vec::new();

        for v in vars.into_iter() {
            if let MalVal::Atom(MalAtom::Sym(sym_name)) = v {
                binds.push(sym_name);
            } else {
                return Err(EvalError::NotASymbol);
            }
        }
        let body = list.remove(0);
        Ok(MalVal::Fn(Box::new(MalFn {
            env: env.clone(),
            body,
            binds,
        })))
    } else {
        Err(EvalError::NotAList)
    }
}

fn handle_do(env: &Environment, mut list: Vec<MalVal>) -> EvalResult<MalVal> {
    list.remove(0);
    let mut ret = MalVal::Atom(MalAtom::Nil);
    for v in list.into_iter() {
        ret = eval(v, env)?;
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_env() -> Environment {
        EnvironmentBuilder::new()
            .with_builtins(builtin::defaults())
            .build()
    }

    #[test]
    fn test_eval() {
        {
            let env = default_env();
            let ast = MalVal::Atom(MalAtom::Sym("undefined_sym".into()));
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::SymbolNotFound("undefined_sym".into()));
        }
        {
            for op in &["+", "-", "*"] {
                let env = default_env();
                let ast = MalVal::Atom(MalAtom::Sym((*op).to_owned()));
                let expected = ast.clone();
                let evaluated = eval(ast, &env).unwrap();

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
                let env = default_env();
                let ast = MalVal::List(vec![atom, MalVal::Atom(MalAtom::Int(2))]);
                let evaluated = eval(ast, &env).unwrap_err();
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
                    let env = default_env();
                    let ast = MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym((*op).to_owned())),
                        atom,
                        MalVal::Atom(MalAtom::Int(2)),
                    ]);
                    let evaluated = eval(ast, &env).unwrap_err();
                    assert!(matches!(evaluated, EvalError::NotANumber))
                }
            }
        }
        {
            for tc in &[("+", 2i64), ("-", -2i64), ("*", 2i64)] {
                let env = default_env();
                let (op, expected) = *tc;
                let ast = MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym(op.to_owned())),
                    MalVal::Atom(MalAtom::Int(2)),
                ]);
                let evaluated = eval(ast, &env).unwrap();

                assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(expected)));
            }
        }
        {
            for tc in &[("+", 9i64), ("-", -5i64), ("*", 24i64)] {
                let env = default_env();
                let (op, expected) = *tc;
                let ast = MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym(op.to_owned())),
                    MalVal::Atom(MalAtom::Int(2)),
                    MalVal::Atom(MalAtom::Int(3)),
                    MalVal::Atom(MalAtom::Int(4)),
                ]);
                let evaluated = eval(ast, &env).unwrap();

                assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(expected)));
            }
        }
    }

    #[test]
    fn test_def() {
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::Atom(MalAtom::Sym("def!".to_string()))]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
                MalVal::Atom(MalAtom::Sym("b".to_string())),
                MalVal::Atom(MalAtom::Sym("c".to_string())),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
                MalVal::Atom(MalAtom::Sym("b".to_string())),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert!(matches!(evaluated, EvalError::SymbolNotFound(_)));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Int(1)),
                MalVal::Atom(MalAtom::Int(2)),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::NotASymbol);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
                MalVal::Atom(MalAtom::Int(2)),
            ]);
            eval(ast, &env).unwrap();

            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("+".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
                MalVal::Atom(MalAtom::Int(10)),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(12)));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("def!".to_string())),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("+".to_string())),
                    MalVal::Atom(MalAtom::Int(2)),
                    MalVal::Atom(MalAtom::Int(3)),
                ]),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(5)));
        }
    }

    #[test]
    fn test_let() {
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::Atom(MalAtom::Sym("let*".to_string()))]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::Atom(MalAtom::Int(1)),
                MalVal::Atom(MalAtom::Int(1)),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::NotAList);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::List(vec![MalVal::Atom(MalAtom::Int(1))]),
                MalVal::Atom(MalAtom::Int(1)),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Int(1)),
                    MalVal::Atom(MalAtom::Int(1)),
                ]),
                MalVal::Atom(MalAtom::Int(1)),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::NotASymbol);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Int(7)),
                ]),
                MalVal::Atom(MalAtom::Sym("a".to_string())),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(7)));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Int(7)),
                    MalVal::Atom(MalAtom::Sym("b".to_string())),
                    MalVal::Atom(MalAtom::Int(13)),
                ]),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("+".to_string())),
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Sym("b".to_string())),
                ]),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(20)));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("let*".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Int(7)),
                    MalVal::Atom(MalAtom::Sym("b".to_string())),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("+".to_string())),
                        MalVal::Atom(MalAtom::Sym("a".to_string())),
                        MalVal::Atom(MalAtom::Int(1)),
                    ]),
                ]),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("+".to_string())),
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Sym("b".to_string())),
                ]),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(15)));
        }
    }

    #[test]
    fn test_fn() {
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::Atom(MalAtom::Sym("fn*".to_string()))]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                MalVal::Atom(MalAtom::False),
                MalVal::Atom(MalAtom::False),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::NotAList);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                MalVal::List(vec![MalVal::Atom(MalAtom::False)]),
                MalVal::Atom(MalAtom::False),
            ]);
            let evaluated = eval(ast, &env).unwrap_err();

            assert_eq!(evaluated, EvalError::NotASymbol);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                MalVal::List(vec![MalVal::Atom(MalAtom::Sym("a".to_string()))]),
                MalVal::Atom(MalAtom::False),
            ]);
            let evaluated = eval(ast, &env).unwrap();

            assert!(matches!(evaluated, MalVal::Fn(_)));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                MalVal::List(vec![MalVal::Atom(MalAtom::Sym("a".to_string()))]),
                MalVal::Atom(MalAtom::False),
            ])]);
            let evaluated = eval(ast, &env).unwrap_err();
            assert_eq!(evaluated, EvalError::InvalidArgs);
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                MalVal::List(vec![]),
                MalVal::Atom(MalAtom::False),
            ])]);
            let evaluated = eval(ast, &env).unwrap();
            assert_eq!(evaluated, MalVal::Atom(MalAtom::False));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                    MalVal::List(vec![MalVal::Atom(MalAtom::Sym("a".to_string()))]),
                    MalVal::Atom(MalAtom::False),
                ]),
                MalVal::Atom(MalAtom::True),
            ]);
            let evaluated = eval(ast, &env).unwrap();
            assert_eq!(evaluated, MalVal::Atom(MalAtom::False));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("fn*".to_string())),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("a".to_string())),
                        MalVal::Atom(MalAtom::Sym("b".to_string())),
                    ]),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("+".to_string())),
                        MalVal::Atom(MalAtom::Sym("a".to_string())),
                        MalVal::Atom(MalAtom::Sym("b".to_string())),
                    ]),
                ]),
                MalVal::Atom(MalAtom::Int(3)),
                MalVal::Atom(MalAtom::Int(4)),
            ]);
            let evaluated = eval(ast, &env).unwrap();
            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(7)));
        }
    }

    #[test]
    fn test_do() {
        {
            let env = default_env();
            let ast = MalVal::List(vec![MalVal::Atom(MalAtom::Sym("do".to_string()))]);
            let evaluated = eval(ast, &env).unwrap();
            assert_eq!(evaluated, MalVal::Atom(MalAtom::Nil));
        }
        {
            let env = default_env();
            let ast = MalVal::List(vec![
                MalVal::Atom(MalAtom::Sym("do".to_string())),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("def!".to_string())),
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("+".to_string())),
                        MalVal::Atom(MalAtom::Int(2)),
                        MalVal::Atom(MalAtom::Int(3)),
                    ]),
                ]),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("+".to_string())),
                    MalVal::Atom(MalAtom::Sym("a".to_string())),
                    MalVal::Atom(MalAtom::Int(4)),
                ]),
            ]);
            let evaluated = eval(ast, &env).unwrap();
            assert_eq!(evaluated, MalVal::Atom(MalAtom::Int(9)));
        }
    }
}
