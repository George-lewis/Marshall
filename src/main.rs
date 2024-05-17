mod codegen;
mod lexer;
mod parser;

fn main() {
    let mut args = std::env::args();

    let input = args.nth(1).expect("missing filename argument");

    let output = args.next();
    let output = output.as_deref().unwrap_or("out.py");

    let input = std::fs::read_to_string(&input).expect("cannot read file");

    let lexer = lexer::Lexer::new(&input);
    let tokens = lexer.lex();

    let parser = parser::Parser::new(&tokens);
    let types = parser.parse();

    let code = codegen::Codegen::new(types).generate();

    std::fs::write(output, code).expect("cannot write file");
}
