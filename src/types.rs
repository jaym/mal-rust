use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum MalVal {
    Atom(MalAtom),
    List(Vec<MalVal>),
    Vector(Vec<MalVal>),
    AssocArray(Vec<MalVal>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum MalAtom {
    Nil,
    True,
    False,
    Sym(String),
    Str(String),
    Int(u64),
}

impl Display for MalVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MalVal::Atom(a) => {
                write!(f, "{}", a)?;
            }
            MalVal::List(seq) => {
                f.write_str("(")?;
                fmt_seq(f, seq)?;
                f.write_str(")")?;
            }
            MalVal::Vector(seq) => {
                f.write_str("[")?;
                fmt_seq(f, seq)?;
                f.write_str("]")?;
            }
            MalVal::AssocArray(seq) => {
                f.write_str("{")?;
                fmt_seq(f, seq)?;
                f.write_str("}")?;
            }
        }
        Ok(())
    }
}

fn fmt_seq<T>(f: &mut std::fmt::Formatter, seq: T) -> std::fmt::Result
where
    T: IntoIterator,
    T::Item: Display,
{
    let mut it = seq.into_iter().peekable();
    while let Some(v) = it.next() {
        write!(f, "{}", v)?;
        if it.peek().is_some() {
            f.write_str(" ")?
        }
    }

    Ok(())
}

impl Display for MalAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MalAtom::Nil => write!(f, "nil"),
            MalAtom::True => write!(f, "true"),
            MalAtom::False => write!(f, "false"),
            MalAtom::Sym(s) => write!(f, "{}", s),
            MalAtom::Str(s) => write!(f, "\"{}\"", s),
            MalAtom::Int(i) => write!(f, "{}", i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_malval() {
        {
            let v = MalVal::List(vec![]);

            assert_eq!(v.to_string(), "()")
        }
        {
            let v = MalVal::List(vec![
                MalVal::Atom(MalAtom::Nil),
                MalVal::Atom(MalAtom::True),
                MalVal::Atom(MalAtom::False),
                MalVal::List(vec![]),
                MalVal::Atom(MalAtom::Sym("hello".into())),
                MalVal::Atom(MalAtom::Str("world".into())),
                MalVal::Atom(MalAtom::Int(123)),
            ]);

            assert_eq!(v.to_string(), "(nil true false () hello \"world\" 123)")
        }

        {
            let v = MalVal::Vector(vec![]);

            assert_eq!(v.to_string(), "[]")
        }
        {
            let v = MalVal::Vector(vec![
                MalVal::Atom(MalAtom::Nil),
                MalVal::Atom(MalAtom::True),
                MalVal::Atom(MalAtom::False),
                MalVal::List(vec![]),
                MalVal::Atom(MalAtom::Sym("hello".into())),
                MalVal::Atom(MalAtom::Str("world".into())),
                MalVal::Atom(MalAtom::Int(123)),
            ]);

            assert_eq!(v.to_string(), "[nil true false () hello \"world\" 123]")
        }

        {
            let v = MalVal::AssocArray(vec![]);

            assert_eq!(v.to_string(), "{}")
        }
    }
}
