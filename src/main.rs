mod lexer;
mod parser;
mod codegen;

fn main() {
    let filename = std::env::args().nth(1).expect("missing filename argument");

    let input = std::fs::read_to_string(&filename).expect("cannot read file");

    let mut lexer = lexer::Lexer::new(&input);

    let tokens = lexer.lex();

    dbg!(&tokens);

    println!("{}", tokens.iter().map(|x| x.to_string() + " ").collect::<String>());

    let mut parser = parser::Parser::new(&tokens);

    let x = parser.parse();

    dbg!(&x);

    let code = codegen::Codegen::new(x).generate();

    println!("{}", code);

    std::fs::write("out.py", code).expect("cannot write file");
}
