use lang::{Atom, Interpreter, Parser};

fn main() {
    let mut exp = String::from("++0102");

    let mut parser = Parser::new(&mut exp);
    let _ = parser.parse();

    let mut interpreter = Interpreter::new(parser.stack);

    let result = interpreter.interpret().unwrap();

    let expected = Atom::Number(3);
    assert_eq!(result, expected);

    println!("Result: {:?}", result);
}
