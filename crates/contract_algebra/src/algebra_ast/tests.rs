use crate::algebra_ast::*;

#[test]
fn test_binop() {
    let x = 3.14;
    let y = 42.;
    assert_eq!(BinaryOperator::Plus.eval(x, y), x + y);
    assert_eq!(BinaryOperator::Minus.eval(x, y), x - y);
    assert_eq!(BinaryOperator::Times.eval(x, y), x * y);
    assert_eq!(BinaryOperator::Div.eval(x, y), x / y);
    assert_eq!(BinaryOperator::Max.eval(x, y), y);
    assert_eq!(BinaryOperator::Min.eval(x, y), x);
    assert_eq!(BinaryOperator::Pow.eval(x, 2.), 9.8596);
}
