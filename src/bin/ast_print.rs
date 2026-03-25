use lex::{AstPrinter, Expr, Literal, Token, TokenType};
use std::rc::Rc;

fn main() {
    let expression = Expr::Binary {
        left: Rc::new(Expr::Unary {
            operator: Token::new(TokenType::Minus, "-", None, 1),
            right: Rc::new(Expr::Literal {
                value: Literal::Number(123.0),
            }),
        }),
        operator: Token::new(TokenType::Star, "*", None, 1),
        right: Rc::new(Expr::Grouping {
            expression: Rc::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    let printer = AstPrinter;
    println!("{}", printer.print(&expression));
}
