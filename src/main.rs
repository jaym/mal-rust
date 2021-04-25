use rustyline;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use types::{MalAtom, MalVal};

mod reader;
mod types;

fn read(input: &str) -> reader::Result<MalVal> {
    let mut v = reader::read_str(input)?;
    if v.len() > 0 {
        Ok(v.swap_remove(0))
    } else {
        Ok(MalVal::Atom(MalAtom::Nil))
    }
}

fn eval(ast: MalVal) -> MalVal {
    ast
}

fn print(res: MalVal) {
    let v = res.to_string();
    println!("{}", v);
}

fn printerr(e: reader::ParseError) {
    println!("error: {}", e);
}

fn rep(input: &str) {
    read(input).map_or_else(
        |e| {
            printerr(e);
        },
        |ast| {
            let res = eval(ast);
            print(res);
        },
    );
}

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        use ValidationResult::{Incomplete, Invalid, Valid};
        let input = ctx.input();

        let result = if let Err(parse_err) = read(input) {
            match parse_err {
                reader::ParseError::EOF => Incomplete,
                _ => Invalid(Some(format!(" ---< {}", parse_err).to_owned())),
            }
        } else {
            Valid(None)
        };

        Ok(result)
    }
}

fn main() {
    let mut rl = rustyline::Editor::new();
    let helper = InputValidator {};
    rl.set_helper(Some(helper));

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rep(&line);
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(_err) => {
                break;
            }
        }
    }
}
