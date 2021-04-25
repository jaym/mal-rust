use crate::types::{MalAtom, MalVal};
use std::iter::Peekable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected EOF")]
    #[allow(clippy::upper_case_acronyms)]
    EOF,
    #[error("Unexpected token {0}")]
    UnxpectedToken(String),
    #[error("Unexpected escapse sequence \\{0}")]
    UnknownEscapeSequence(char),
    #[error("Unexpected newline")]
    UnexpectedNewline,
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub fn read_str(input: &str) -> Result<Vec<MalVal>> {
    let tokens = tokenize(input)?;
    let mut it = tokens.into_iter().peekable();
    let mut ret = Vec::new();
    while it.peek().is_some() {
        if let Some(f) = read_form(&mut it)? {
            ret.push(f)
        }
    }
    Ok(ret)
}

fn read_form<I>(it: &mut Peekable<I>) -> Result<Option<MalVal>>
where
    I: Iterator<Item = Token>,
{
    if let Some(tok) = it.peek() {
        match tok {
            Token::SingleQuote => {
                unimplemented!()
            }
            Token::Tick => {
                unimplemented!()
            }
            Token::LeftParen => {
                it.next();
                let seq = read_seq(it, Token::RightParen)?;
                Ok(Some(MalVal::List(seq)))
            }
            Token::LeftBracket => {
                it.next();
                let seq = read_seq(it, Token::RightBracket)?;
                Ok(Some(MalVal::Vector(seq)))
            }
            Token::LeftCurly => {
                it.next();
                let seq = read_seq(it, Token::RightCurly)?;
                Ok(Some(MalVal::AssocArray(seq)))
            }
            _ => Ok(read_atom(it)?),
        }
    } else {
        Ok(None)
    }
}

fn read_seq<I>(it: &mut Peekable<I>, until: Token) -> Result<Vec<MalVal>>
where
    I: Iterator<Item = Token>,
{
    let mut res = Vec::new();
    while let Some(v) = it.peek() {
        if *v == until {
            it.next();
            return Ok(res);
        }
        if let Some(f) = read_form(it)? {
            res.push(f)
        }
    }
    Err(ParseError::EOF)
}

fn read_atom<I>(it: &mut Peekable<I>) -> Result<Option<MalVal>>
where
    I: Iterator<Item = Token>,
{
    it.next().map_or(Ok(None), |tok| match tok {
        Token::Int(i) => Ok(Some(MalVal::Atom(MalAtom::Int(i)))),
        Token::Str(s) => Ok(Some(MalVal::Atom(MalAtom::Str(s)))),
        Token::Lit(l) => {
            let s: &str = &l;
            let atom = match s {
                "nil" => MalAtom::Nil,
                "true" => MalAtom::True,
                "false" => MalAtom::False,
                _ => MalAtom::Sym(l),
            };
            Ok(Some(MalVal::Atom(atom)))
        }
        _ => Err(ParseError::UnxpectedToken(format!("{:?}", tok))),
    })
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
    Int(i64),
    Str(String),
    Lit(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>> {
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
                let s = read_string(&mut it)?;
                result.push(Token::Str(s));
            }
            ';' => {
                let _ = read_comment(&mut it);
                //result.push(Token::Comment(comment));
            }
            '-' => {
                if let Some(num) = read_number(&mut it, None) {
                    result.push(Token::Int(-num))
                } else {
                    let lit = read_literal(&mut it, c);
                    result.push(Token::Lit(lit));
                }
            }
            '0'..='9' => {
                let num = read_number(&mut it, Some(c));
                result.push(Token::Int(num.unwrap()))
            }
            _ => {
                if !(c.is_whitespace() || c == ',') {
                    let lit = read_literal(&mut it, c);
                    result.push(Token::Lit(lit));
                }
            }
        }
    }
    Ok(result)
}

fn read_number<I: Iterator<Item = char>>(
    it: &mut Peekable<I>,
    first_digit: Option<char>,
) -> Option<i64> {
    let (mut v, mut num_found) =
        first_digit.map_or((0i64, 0), |c| (c.to_digit(10).unwrap() as i64, 1));

    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' => {
                it.next();
                let num = c.to_digit(10).unwrap() as i64;
                v = v * 10 + num;
                num_found += 1;
            }
            _ => break,
        }
    }
    if num_found > 0 {
        Some(v)
    } else {
        None
    }
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

fn read_string<I: Iterator<Item = char>>(it: &mut Peekable<I>) -> Result<String> {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        it.next();
        match c {
            '\n' => return Err(ParseError::UnexpectedNewline),
            '\\' => {
                if let Some(nc) = it.peek() {
                    match nc {
                        '"' => s.push('"'),
                        'n' => s.push('\n'),
                        '\\' => s.push('\\'),
                        c => return Err(ParseError::UnknownEscapeSequence(*c)),
                    }
                    it.next();
                } else {
                    return Err(ParseError::EOF);
                }
            }
            '"' => return Ok(s),
            _ => {
                s.push(c);
            }
        }
    }
    Err(ParseError::EOF)
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
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        {
            let s = " , \n  \t ";
            let v = tokenize(&s.to_string()).unwrap();
            assert_eq!(v, vec![]);
        }

        {
            let s = "  ( ,,, ) [ ]}  \n  \t {";
            let v = tokenize(&s.to_string()).unwrap();
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
            let v = tokenize(&s.to_string()).unwrap();
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
            let s = "  (+ 0 12 345 6789 -1 -12 -123)";
            let v = tokenize(&s.to_string()).unwrap();
            assert_eq!(
                v,
                vec![
                    Token::LeftParen,
                    Token::Lit("+".into()),
                    Token::Int(0),
                    Token::Int(12),
                    Token::Int(345),
                    Token::Int(6789),
                    Token::Int(-1),
                    Token::Int(-12),
                    Token::Int(-123),
                    Token::RightParen,
                ]
            );
        }

        {
            let s = "  (+ \"asd\\\"f\")";
            let v = tokenize(&s.to_string()).unwrap();
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
            let s = "\"a\\nb\"";
            let v = tokenize(&s.to_string()).unwrap();
            assert_eq!(v, vec![Token::Str("a\nb".into()),]);
        }
        {
            let s = "\"a\\\\b\"";
            let v = tokenize(&s.to_string()).unwrap();
            assert_eq!(v, vec![Token::Str("a\\b".into()),]);
        }

        {
            let s = " ; ()[]}\t{\n()";
            let v = tokenize(&s.to_string()).unwrap();
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
            let v = read_str(&s).unwrap();
            assert_eq!(v, vec![],);
        }
        {
            let s = r#"
            (println "hello")
            "#;
            let v = read_str(&s).unwrap();
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
            let v = read_str(&s).unwrap();
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
            let v = read_str(&s).unwrap();
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

        {
            let s = r#"
            (nil true false)
            "#;
            let v = read_str(&s).unwrap();
            assert_eq!(
                *v.first().unwrap(),
                MalVal::List(vec![
                    MalVal::Atom(MalAtom::Nil),
                    MalVal::Atom(MalAtom::True),
                    MalVal::Atom(MalAtom::False),
                ])
            );
        }
    }
}
