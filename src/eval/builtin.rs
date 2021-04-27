use crate::types::{EvalError, EvalResult, MalAtom, MalVal, NativeFn};
use std::collections::HashMap;

pub fn defaults() -> HashMap<String, NativeFn> {
    let mut h: HashMap<String, NativeFn> = HashMap::new();
    h.insert("+".to_owned(), add);
    h.insert("-".to_owned(), sub);
    h.insert("*".to_owned(), mul);
    h.insert("=".to_owned(), eq);
    h.insert(">".to_owned(), gt);
    h.insert(">=".to_owned(), gte);
    h.insert("<".to_owned(), lt);
    h.insert("<=".to_owned(), lte);
    h.insert("list".to_owned(), list);
    h.insert("list?".to_owned(), is_list);
    h.insert("empty?".to_owned(), is_empty);
    h.insert("count".to_owned(), count);
    h
}

fn add(args: Vec<MalVal>) -> EvalResult<MalVal> {
    let mut acc: i64 = 0;
    for v in args.into_iter() {
        if let MalVal::Atom(MalAtom::Int(num)) = v {
            acc += num;
        } else {
            return Err(EvalError::NotANumber);
        }
    }
    Ok(MalVal::Atom(MalAtom::Int(acc)))
}

fn sub(args: Vec<MalVal>) -> EvalResult<MalVal> {
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
}

fn mul(args: Vec<MalVal>) -> EvalResult<MalVal> {
    let mut acc: i64 = 1;
    for v in args.into_iter() {
        if let MalVal::Atom(MalAtom::Int(num)) = v {
            acc *= num;
        } else {
            return Err(EvalError::NotANumber);
        }
    }
    Ok(MalVal::Atom(MalAtom::Int(acc)))
}

fn eq(args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 2 {
        Err(EvalError::InvalidArgs)
    } else {
        Ok(MalVal::Atom(if args[0] == args[1] {
            MalAtom::True
        } else {
            MalAtom::False
        }))
    }
}

fn gt(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 2 {
        Err(EvalError::InvalidArgs)
    } else {
        let arg0 = into_int(args.remove(0))?;
        let arg1 = into_int(args.remove(0))?;

        Ok(MalVal::Atom((arg0 > arg1).into()))
    }
}

fn gte(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 2 {
        Err(EvalError::InvalidArgs)
    } else {
        let arg0 = into_int(args.remove(0))?;
        let arg1 = into_int(args.remove(0))?;

        Ok(MalVal::Atom((arg0 >= arg1).into()))
    }
}

fn lt(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 2 {
        Err(EvalError::InvalidArgs)
    } else {
        let arg0 = into_int(args.remove(0))?;
        let arg1 = into_int(args.remove(0))?;

        Ok(MalVal::Atom((arg0 < arg1).into()))
    }
}

fn lte(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 2 {
        Err(EvalError::InvalidArgs)
    } else {
        let arg0 = into_int(args.remove(0))?;
        let arg1 = into_int(args.remove(0))?;

        Ok(MalVal::Atom((arg0 <= arg1).into()))
    }
}

#[allow(clippy::clippy::unnecessary_wraps)]
fn list(args: Vec<MalVal>) -> EvalResult<MalVal> {
    Ok(MalVal::List(args))
}

fn count(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 1 {
        Err(EvalError::InvalidArgs)
    } else if let MalVal::List(list) = args.remove(0) {
        Ok(MalVal::Atom(MalAtom::Int(list.len() as i64)))
    } else {
        Err(EvalError::NotAList)
    }
}

fn is_empty(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 1 {
        Err(EvalError::InvalidArgs)
    } else if let MalVal::List(list) = args.remove(0) {
        if list.is_empty() {
            Ok(MalVal::Atom(MalAtom::True))
        } else {
            Ok(MalVal::Atom(MalAtom::False))
        }
    } else {
        Err(EvalError::NotAList)
    }
}

fn is_list(mut args: Vec<MalVal>) -> EvalResult<MalVal> {
    if args.len() != 1 {
        Err(EvalError::InvalidArgs)
    } else if let MalVal::List(_) = args.remove(0) {
        Ok(MalVal::Atom(MalAtom::True))
    } else {
        Ok(MalVal::Atom(MalAtom::False))
    }
}

pub fn into_int(v: MalVal) -> EvalResult<i64> {
    if let MalVal::Atom(MalAtom::Int(i)) = v {
        Ok(i)
    } else {
        Err(EvalError::NotANumber)
    }
}
