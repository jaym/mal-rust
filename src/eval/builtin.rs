use std::collections::HashMap;

use crate::types::{EvalError, EvalResult, MalAtom, MalVal, NativeFn};

pub fn defaults() -> HashMap<String, NativeFn> {
    let mut h: HashMap<String, NativeFn> = HashMap::new();
    h.insert("+".to_owned(), add);
    h.insert("-".to_owned(), sub);
    h.insert("*".to_owned(), mul);
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
