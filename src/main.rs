use eval::builtin;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use types::{
    env::{Environment, EnvironmentBuilder},
    EvalResult, MalAtom, MalVal,
};

mod eval;
mod reader;
mod types;

fn read(input: &str) -> reader::Result<MalVal> {
    let mut v = reader::read_str(input)?;
    if !v.is_empty() {
        Ok(v.swap_remove(0))
    } else {
        Ok(MalVal::Atom(MalAtom::Nil))
    }
}

fn eval(ast: MalVal, env: &mut Environment) -> EvalResult<MalVal> {
    eval::eval(ast, env)
}

fn print(res: MalVal) {
    let v = res.to_string();
    println!("{}", v);
}

fn print_err<T: std::fmt::Display>(e: T) {
    println!("error: {}", e);
}

fn rep(input: &str, env: &mut Environment) {
    read(input).map_or_else(
        |e| {
            print_err(e);
        },
        |ast| {
            eval(ast, env).map_or_else(print_err, print);
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
                _ => Invalid(Some(format!(" ---< {}", parse_err))),
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
    let mut env = EnvironmentBuilder::new()
        .with_builtins(builtin::defaults())
        .build();
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rep(&line, &mut env);
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
