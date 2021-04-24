use crate::types::{MalAtom, MalVal};

use std::iter::Peekable;

pub fn read_str<'a>(input: &'a str) -> Vec<MalVal> {
    let tokens = tokenize(input);
    let mut it = tokens.into_iter().peekable();
    let mut ret = Vec::new();
    while let Some(_) = it.peek() {
        let f = read_form(&mut it);
        ret.push(f);
    }
    ret
}

fn read_form<'a, I>(it: &mut Peekable<I>) -> MalVal
where
    I: Iterator<Item = Token>,
{
    let tok = it.peek().unwrap();
    match tok {
        Token::SingleQuote => {
            unimplemented!()
        }
        Token::Tick => {
            unimplemented!()
        }
        Token::LeftParen => {
            it.next();
            let seq = read_seq(it, Token::RightParen);
            MalVal::List(seq)
        }
        Token::LeftBracket => {
            it.next();
            let seq = read_seq(it, Token::RightBracket);
            MalVal::Vector(seq)
        }
        Token::LeftCurly => {
            it.next();
            let seq = read_seq(it, Token::RightCurly);
            MalVal::AssocArray(seq)
        }
        _ => read_atom(it),
    }
}

fn read_seq<'a, I>(it: &mut Peekable<I>, until: Token) -> Vec<MalVal>
where
    I: Iterator<Item = Token>,
{
    let mut res = Vec::new();
    while let Some(v) = it.peek() {
        if *v == until {
            it.next();
            return res;
        }
        let form = read_form(it);
        res.push(form);
    }
    panic!("Unexpected EOF")
}

fn read_atom<'a, I>(it: &mut Peekable<I>) -> MalVal
where
    I: Iterator<Item = Token>,
{
    let tok = it.next().unwrap();
    match tok {
        Token::Int(i) => MalVal::Atom(MalAtom::Int(i)),
        Token::Str(s) => MalVal::Atom(MalAtom::Str(s)),
        Token::Lit(l) => MalVal::Atom(MalAtom::Sym(l)),
        _ => panic!("Unexpected token {:?}", tok),
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftCurly,
    RightCurly,
    SingleQuote,
    Tick,
    Int(u64),
    Comment(String),
    Str(String),
    Lit(String),
}

fn tokenize<'a>(input: &'a str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut it = input.chars().peekable();

    while let Some(c) = it.next() {
        match c {
            '(' => result.push(Token::LeftParen),
            ')' => result.push(Token::RightParen),
            '[' => result.push(Token::LeftBracket),
            ']' => result.push(Token::RightBracket),
            '{' => result.push(Token::LeftCurly),
            '}' => result.push(Token::RightCurly),
            '\'' => result.push(Token::SingleQuote),
            '`' => result.push(Token::Tick),
            '"' => {
                let s = read_string(&mut it);
                result.push(Token::Str(s));
            }
            ';' => {
                let _ = read_comment(&mut it);
                //result.push(Token::Comment(comment));
            }
            '0'..='9' => {
                let num = read_number(&mut it, c);
                result.push(Token::Int(num))
            }
            _ => {
                if !c.is_whitespace() {
                    let lit = read_literal(&mut it, c);
                    result.push(Token::Lit(lit));
                }
            }
        }
    }
    result
}

fn read_number<I: Iterator<Item = char>>(it: &mut Peekable<I>, first_digit: char) -> u64 {
    let mut v = first_digit.to_digit(10).unwrap() as u64;

    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' => {
                it.next();
                let num = c.to_digit(10).unwrap() as u64;
                v = v * 10 + num;
            }
            _ => break,
        }
    }
    v
}

fn read_comment<I: Iterator<Item = char>>(it: &mut Peekable<I>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        it.next();
        match c {
            '\n' => {
                break;
            }
            _ => {
                s.push(c);
            }
        }
    }
    s
}

fn read_string<I: Iterator<Item = char>>(it: &mut Peekable<I>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        it.next();
        match c {
            '\n' => panic!("unexpected newline"),
            '\\' => {
                if it.peek() == Some(&'"') {
                    s.push('"');
                    it.next();
                } else {
                    panic!("unsupported escape sequence");
                }
            }
            '"' => return s,
            _ => {
                s.push(c);
            }
        }
    }
    panic!("unexpected EOF");
}

fn read_literal<I: Iterator<Item = char>>(it: &mut Peekable<I>, first_char: char) -> String {
    let mut s = String::new();
    s.push(first_char);
    while let Some(&c) = it.peek() {
        match c {
            '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | ';' | '`' | '~' => break,
            _ => {
                if c.is_whitespace() {
                    break;
                }
                s.push(c);
                it.next();
            }
        }
    }
    return s;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        {
            let s = "  \n  \t ";
            let v = tokenize(&s.to_string());
            assert_eq!(v, vec![]);
        }

        {
            let s = "  (  ) [ ]}  \n  \t {";
            let v = tokenize(&s.to_string());
            assert_eq!(
                v,
                vec![
                    Token::LeftParen,
                    Token::RightParen,
                    Token::LeftBracket,
                    Token::RightBracket,
                    Token::RightCurly,
                    Token::LeftCurly
                ]
            );
        }

        {
            let s = "  (+ asdf)";
            let v = tokenize(&s.to_string());
            assert_eq!(
                v,
                vec![
                    Token::LeftParen,
                    Token::Lit("+".into()),
                    Token::Lit("asdf".into()),
                    Token::RightParen,
                ]
            );
        }

        {
            let s = "  (+ 0 12 345 6789)";
            let v = tokenize(&s.to_string());
            assert_eq!(
                v,
                vec![
                    Token::LeftParen,
                    Token::Lit("+".into()),
                    Token::Int(0),
                    Token::Int(12),
                    Token::Int(345),
                    Token::Int(6789),
                    Token::RightParen,
                ]
            );
        }

        {
            let s = "  (+ \"asd\\\"f\")";
            let v = tokenize(&s.to_string());
            assert_eq!(
                v,
                vec![
                    Token::LeftParen,
                    Token::Lit("+".into()),
                    Token::Str("asd\"f".into()),
                    Token::RightParen,
                ]
            );
        }

        {
            let s = " ; ()[]}\t{\n()";
            let v = tokenize(&s.to_string());
            assert_eq!(
                v,
                vec![
                    // Token::Comment(" ()[]}\t{".into()),
                    Token::LeftParen,
                    Token::RightParen,
                ]
            );
        }
    }

    #[test]
    fn test_read_str() {
        {
            let s = r#"
            "#;
            let v = read_str(&s);
            assert_eq!(v, vec![],);
        }
        {
            let s = r#"
            (println "hello")
            "#;
            let v = read_str(&s);
            assert_eq!(
                *v.first().unwrap(),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("println".into())),
                    MalVal::Atom(MalAtom::Str("hello".into()))
                ])
            );
        }

        {
            let s = r#"
            (println "hello")
            (print-line "world")
            "#;
            let v = read_str(&s);
            assert_eq!(
                v,
                vec![
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("println".into())),
                        MalVal::Atom(MalAtom::Str("hello".into()))
                    ]),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("print-line".into())),
                        MalVal::Atom(MalAtom::Str("world".into()))
                    ]),
                ]
            );
        }

        {
            let s = r#"
            (fun1! 2 "hello" 
                (fun2? 3 "world"))
            "#;
            let v = read_str(&s);
            assert_eq!(
                v,
                vec![MalVal::List(vec![
                    MalVal::Atom(MalAtom::Sym("fun1!".into())),
                    MalVal::Atom(MalAtom::Int(2)),
                    MalVal::Atom(MalAtom::Str("hello".into())),
                    MalVal::List(vec![
                        MalVal::Atom(MalAtom::Sym("fun2?".into())),
                        MalVal::Atom(MalAtom::Int(3)),
                        MalVal::Atom(MalAtom::Str("world".into())),
                    ])
                ]),]
            );
        }
    }
}
