include!("grammar.rs"); // auto-generated by lalrpop

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{Expr, Op};

    fn s(x: &str) -> String {
        String::from(x)
    }

    #[test]
    fn test_parsing_number() {
        let expected = Box::new(Expr::Number(22.0));
        assert_eq!(parse_Expr("22").unwrap(), expected);
        assert_eq!(parse_Expr("(22)").unwrap(), expected);
        assert_eq!(parse_Expr("((((((22))))))").unwrap(), expected);
    }
    #[test]
    fn test_parsing_expressions() {
        let expected = Box::new(Expr::Binary(Box::new(Expr::Number(22.0)),
                                             Op::Plus,
                                             Box::new(Expr::Binary(
                                                 Box::new(Expr::Name(s("foo"))),
                                                 Op::Times,
                                                 Box::new(Expr::Number(2.0))))));
        assert_eq!(parse_Expr("22+foo*2").unwrap(), expected);
    }
    #[test]
    fn test_parsing_names() {
        let expected = Box::new(Expr::Name(s("foo")));
        assert_eq!(parse_Expr("foo").unwrap(), expected);
        assert_eq!(parse_Expr("(foo)").unwrap(), expected);
    }
    #[test]
    fn test_call_parsing() {
        let expected = Box::new(Expr::Call(Box::new(Expr::Name(s("foo"))), vec![]));
        assert_eq!(parse_Expr("foo()").unwrap(), expected);
        let expected = Box::new(Expr::Call(Box::new(Expr::Name(s("foo"))), vec![Box::new(Expr::Number(1.0))]));
        assert_eq!(parse_Expr("foo(1)").unwrap(), expected);
        let expected = Box::new(Expr::Call(Box::new(Expr::Name(s("foo"))), vec![Box::new(Expr::Number(1.0)),
                                                                     Box::new(Expr::Number(2.0)),
                                                                     Box::new(Expr::Number(3.0))]));
        assert_eq!(parse_Expr("foo(1, 2, 3)").unwrap(), expected);
        let expected = Box::new(Expr::Call(Box::new(Expr::Name(s("foo"))), vec![Box::new(Expr::Binary(Box::new(Expr::Number(1.0)), Op::Plus, Box::new(Expr::Number(2.0))))]));
        assert_eq!(parse_Expr("foo(1 + 2)").unwrap(), expected);
        let expected = Box::new(Expr::Call(Box::new(Expr::Name(s("foo"))), vec![Box::new(Expr::Binary(Box::new(Expr::Number(1.0)), Op::Plus, Box::new(Expr::Number(2.0)))),
                                                                     Box::new(Expr::Number(3.0))]));
        assert_eq!(parse_Expr("foo(1 + 2, 3)").unwrap(), expected);
        let expected = Box::new(Expr::Call(Box::new(Expr::Binary(Box::new(Expr::Number(1.0)), Op::Plus, Box::new(Expr::Number(2.0)))), vec![Box::new(Expr::Binary(Box::new(Expr::Number(1.0)), Op::Plus, Box::new(Expr::Number(2.0)))),
                                                                     Box::new(Expr::Number(3.0))]));
        assert_eq!(parse_Expr("(1.0 + 2)(1 + 2, 3)").unwrap(), expected);
    }
    #[test]
    fn test_lambda_parsing() {
        let expected = Box::new(Expr::Lambda(vec![s("x"), s("y")],
                                             Box::new(Expr::Binary(
                                                      Box::new(Expr::Name(s("x"))),
                                                      Op::Plus,
                                                      Box::new(Expr::Number(1.0)
                                                  )))));
        assert_eq!(parse_Expr(r"|x, y| -> (x + 1)").unwrap(), expected);
        let expected = Box::new(Expr::Lambda(vec![s("x")],
                                             Box::new(Expr::Binary(
                                                      Box::new(Expr::Name(s("x"))),
                                                      Op::Plus,
                                                      Box::new(Expr::Number(1.0)
                                                  )))));
        assert_eq!(parse_Expr(r"x -> (x + 1.0)").unwrap(), expected);
        assert_eq!(parse_Expr(r"|x| -> x + 1").unwrap(), expected);
    }
    #[test]
    fn test_pipe_parsing() {
        let expected = Box::new(Expr::Binary(
                            Box::new(Expr::Binary(
                                Box::new(Expr::Call(Box::new(Expr::Name(s("range"))), vec![Box::new(Expr::Number(10.0))])),
                                Op::Pipe,
                                Box::new(Expr::Call(Box::new(Expr::Name(s("map"))), vec![Box::new(Expr::Lambda(vec![s("x")], Box::new(Expr::Binary(Box::new(Expr::Name(s("x"))), Op::Times, Box::new(Expr::Name(s("x")))))))]))
                            )),
                            Op::Pipe,
                            Box::new(Expr::Call(Box::new(Expr::Name(s("foreach"))), vec![Box::new(Expr::Lambda(vec![s("x")], Box::new(Expr::Call(Box::new(Expr::Name(s("display"))), vec![Box::new(Expr::Name(s("x")))]))))]))));
        let got = parse_Expr(r"range(10) | map(x -> x * x) | foreach(x -> display(x))").unwrap();
        assert_eq!(got, expected);
    }
    #[test]
    fn test_assignment_parsing() {
        let expected = Box::new(Expr::Assignment(s("spam"), Box::new(Expr::Number(1.0))));
        let got = parse_Expr("spam := 1").unwrap();
        assert_eq!(got, expected);
    }
    #[test]
    fn test_push_parsing() {
        let expected = Box::new(Expr::Push(Box::new(Expr::Number(1.0))));
        let got = parse_Expr("push 1").unwrap();
        assert_eq!(got, expected);
    }
    #[test]
    fn test_block_parsing() {
        let expected = Box::new(Expr::Block(
                                vec![Box::new(Expr::Assignment(s("spam"), Box::new(Expr::Number(1.0)))),
                                     Box::new(Expr::Push(Box::new(Expr::Number(1.0))))]));
        let got = parse_Expr(r"{spam := 1; push 1}").unwrap();
        assert_eq!(expected, got);
    }
    #[test]
    fn test_if_else_parsing() {
        let expected = Box::new(Expr::If(Box::new(Expr::Number(1.0)), Box::new(Expr::Number(2.0)), Box::new(Expr::Number(3.0))));
        let got = parse_Expr(r"if 1 then 2 else 3").unwrap();
        assert_eq!(expected, got);
        let expected = Box::new(Expr::Lambda(vec![s("x")],
                                             Box::new(
                                                 Expr::If(
                                                     Box::new(Expr::Number(1.0)),
                                                     Box::new(Expr::Number(2.0)),
                                                     Box::new(Expr::Number(3.0))
                                                 )
                                             )
                                         ));
        let got = parse_Expr(r"x -> if 1 then 2 else 3").unwrap();
        assert_eq!(expected, got);
    }
    #[test]
    fn test_while_parsing() {
        let expected = Box::new(Expr::While(Box::new(Expr::Number(1.0)), Box::new(Expr::Number(2.0))));
        let got = parse_Expr(r"while 1 do 2").unwrap();
        assert_eq!(expected, got);
    }
    #[test]
    fn test_program_parsing() {
        let expected = r#"[Definition { prototype: Prototype { name: "add", args: ["x"] }, body: Binary(Name("x"), Plus, Number(1)) }, Definition { prototype: Prototype { name: "bar", args: ["y"] }, body: Binary(Name("y"), Times, Number(2)) }]"#;
        let got = format!("{:?}", parse_Program(r"add(x) => x + 1
        bar(y) => y * 2").unwrap());
        assert_eq!(got, expected);
    }
}
