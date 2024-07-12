use lang::Parser;

fn main() {
    let mut exp = String::from("++0102");

    let mut _parser = Parser::from(&mut exp).parse();

    // let mut interpreter = Interpreter::new(parser.stack());

    // let result = interpreter.interpret().unwrap();

    // let expected = Atom::Number(3);
    // assert_eq!(result, expected);

    // println!("Result: {:?}", result);

    std::thread::sleep(std::time::Duration::from_secs(100));
}
