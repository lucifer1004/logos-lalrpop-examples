use lalrpop_util::{lalrpop_mod, ParseError};
use logos::{Lexer, Logos};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

lalrpop_mod!(expr, "/bin/calculator.rs");

#[derive(Debug, Clone, PartialEq)]
pub enum WrappedInt {
    Int(i64),
    Err(String),
}

fn handle_int(lex: &mut Lexer<Token>) -> WrappedInt {
    let s = lex.slice().to_string();

    match s.parse::<i64>() {
        Ok(i) => WrappedInt::Int(i),
        Err(_) => WrappedInt::Err(format!("{} is too large", s)),
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum Token {
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[regex("[0-9]+", handle_int)]
    Integer(WrappedInt),

    #[error]
    #[regex(r"[ \r\v\t\n\f]+", logos::skip)]
    RawError,

    Error(String), // This is not used by logos itself
}

struct Bridge<'source> {
    lexer: Lexer<'source, Token>,
}

impl<'source> Bridge<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
        }
    }
}

impl<'source> Iterator for Bridge<'source> {
    type Item = (usize, Token, usize);
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|token| {
            let span = self.lexer.span();
            (
                span.start,
                match token {
                    Token::RawError => Token::Error(self.lexer.slice().to_string()),
                    _ => token,
                },
                span.end,
            )
        })
    }
}

fn main() -> Result<()> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let parser = expr::ExprParser::new();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str());
                match parser.parse(Bridge::new(line.as_str())) {
                    Ok(result) => println!("{}", result),
                    Err(err) => println!(
                        "Error: {}",
                        match err {
                            ParseError::User { error } => error,
                            ParseError::UnrecognizedToken { token, expected } => format!(
                                "Unexpected token {}, expected: {}",
                                match token.1 {
                                    Token::Error(tok) => tok,
                                    _ => format!("{:?}", token.1),
                                },
                                expected.join(", ")
                            ),
                            _ => format!("{:?}", err),
                        }
                    ),
                }
            }
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
