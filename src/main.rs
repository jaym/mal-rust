use rustyline::error::ReadlineError;
use rustyline::Editor;
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

fn main() {
    let mut rl = Editor::<()>::new();

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
