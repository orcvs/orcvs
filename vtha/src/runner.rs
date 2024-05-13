//! In this example we build an [S-expression](https://en.wikipedia.org/wiki/S-expression)
//! parser and tiny [lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language)) interpreter.
//! Lisp is a simple type of language made up of Atoms and Lists, forming easily parsable trees.

mod parser;

// Application(
//     IfElse(
//         Application(
//             Constant(BuiltIn(Equal)),
//             [
//                 Application(
//                     Constant(BuiltIn(Plus)),
//                     [
//                         Constant(Num(3)),
//                         Application(
//                             Constant(BuiltIn(Divide)),
//                             [
//                                 Constant(Num(9)), Constant(Num(3))
//                             ]
//                         )
//                     ]
//                 ),
//                 Application(
//                     Constant(BuiltIn(Times)),
//                     [
//                         Constant(Num(2)), Constant(Num(3))
//                     ]
//                 )
//             ]
//         ),
//         Constant(BuiltIn(Times)),
//         Constant(BuiltIn(Divide))
//     ),
//     [
//         Constant(Num(456)), Constant(Num(123))
//     ]
// );

fn main() {
    let expression_1 = "((if (= (+ 3 (/ 9 3))
         (* 2 3))
     *
     /)
  456 123)";
    println!(
        "\"{}\"\nevaled gives us: {:?}",
        expression_1,
        parser::eval_from_str(expression_1)
    );
}
