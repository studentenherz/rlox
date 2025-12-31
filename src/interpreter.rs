use std::fmt::Display;

use crate::common::Span;
use crate::{expressions::*, values::Value};

#[derive(Debug, PartialEq)]
enum ErrorKind {
    TypeError,
}

#[derive(Debug)]
pub struct RuntimeError {
    kind: ErrorKind,
    reason: String,
    span: Span,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] {:?}: {}",
            self.span.line, self.kind, self.reason
        )
    }
}

impl RuntimeError {
    pub fn new_type_error(reason: &str, span: Span) -> Self {
        Self {
            kind: ErrorKind::TypeError,
            reason: reason.to_string(),
            span,
        }
    }
}

type RuntimeResult = Result<Value, RuntimeError>;

pub trait Interpret {
    fn interpret(&self) -> RuntimeResult;
}

impl Interpret for Expr {
    fn interpret(&self) -> RuntimeResult {
        match &self.kind {
            ExprKind::Literal { value } => Ok(Value::from_literal(value)),
            ExprKind::Grouping { expression } => expression.interpret(),
            ExprKind::Unary { operator, right } => {
                let right_value = right.interpret()?;

                match operator {
                    UnaryOperator::Minus => {
                        if let Value::Number(number) = right_value {
                            Ok(Value::Number(-number))
                        } else {
                            Err(RuntimeError::new_type_error(
                                &format!(
                                    "unsupported operand type: {} '{}'",
                                    operator,
                                    right_value.type_name()
                                ),
                                self.span.clone(),
                            ))
                        }
                    }
                    UnaryOperator::Bang => Ok(Value::Boolean(!bool::from(right_value))),
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = left.interpret()?;
                let right_value = right.interpret()?;

                match operator {
                    BinaryOperator::Comma => Ok(right_value),
                    BinaryOperator::Minus
                    | BinaryOperator::Plus
                    | BinaryOperator::Slash
                    | BinaryOperator::Star => try_arithmetic(
                        left_value,
                        right_value,
                        operator.clone(),
                        Span::union(&left.span, &right.span),
                    ),
                    BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual => try_compare(
                        left_value,
                        right_value,
                        operator.clone(),
                        Span::union(&left.span, &right.span),
                    ),
                    BinaryOperator::EqualEqual => Ok(Value::Boolean(left_value == right_value)),
                    BinaryOperator::BangEqual => Ok(Value::Boolean(!(left_value == right_value))),
                }
            }
            ExprKind::Ternary {
                left,
                middle,
                right,
            } => {
                let left_value = left.interpret()?;

                if bool::from(left_value) {
                    middle.interpret()
                } else {
                    right.interpret()
                }
            }
        }
    }
}

fn try_arithmetic(
    left: Value,
    right: Value,
    operator: BinaryOperator,
    span: Span,
) -> RuntimeResult {
    match (&left, &right) {
        (Value::Number(inner_left), Value::Number(inner_right)) => {
            let result = match operator {
                BinaryOperator::Minus => inner_left - inner_right,
                BinaryOperator::Plus => inner_left + inner_right,
                BinaryOperator::Slash => inner_left / inner_right,
                BinaryOperator::Star => inner_left * inner_right,
                _ => unreachable!(),
            };
            Ok(Value::Number(result))
        }
        (Value::String(_), _) | (_, Value::String(_)) if operator == BinaryOperator::Plus => {
            let left = match left {
                Value::String(string) => string,
                val => val.to_string(),
            };

            let right = match right {
                Value::String(string) => string,
                val => val.to_string(),
            };

            Ok(Value::String(format!("{}{}", left, right)))
        }
        _ => Err(RuntimeError::new_type_error(
            &format!(
                "unsupported operand type(s): '{}' {} '{}'",
                left.type_name(),
                operator,
                right.type_name()
            ),
            span,
        )),
    }
}

fn try_compare(left: Value, right: Value, operator: BinaryOperator, span: Span) -> RuntimeResult {
    if let (Value::Number(inner_left), Value::Number(inner_right)) = (&left, &right) {
        let result = match operator {
            BinaryOperator::Less => inner_left < inner_right,
            BinaryOperator::LessEqual => inner_left <= inner_right,
            BinaryOperator::Greater => inner_left > inner_right,
            BinaryOperator::GreaterEqual => inner_left >= inner_right,
            _ => unreachable!(),
        };

        Ok(Value::Boolean(result))
    } else {
        Err(RuntimeError::new_type_error(
            &format!(
                "unsupported operand type(s): '{}' {} '{}'",
                left.type_name(),
                operator,
                right.type_name()
            ),
            span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    use super::*;

    #[test]
    fn interpret_literal_expression() {
        let nil = Parser::parse("nil").unwrap().interpret();
        let true_value = Parser::parse("true").unwrap().interpret();
        let false_value = Parser::parse("false").unwrap().interpret();
        let number = Parser::parse("1").unwrap().interpret();
        let string = Parser::parse("\"hello\"").unwrap().interpret();

        assert_eq!(nil.unwrap(), Value::Nil);
        assert_eq!(true_value.unwrap(), Value::Boolean(true));
        assert_eq!(false_value.unwrap(), Value::Boolean(false));
        assert_eq!(number.unwrap(), Value::Number(1f64));
        assert_eq!(string.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn interpret_unary_expression() {
        let minus_one = Parser::parse("-1").unwrap().interpret();
        let false_value = Parser::parse("!true").unwrap().interpret();
        let true_value = Parser::parse("!false").unwrap().interpret();

        assert_eq!(minus_one.unwrap(), Value::Number(-1f64));
        assert_eq!(false_value.unwrap(), Value::Boolean(false));
        assert_eq!(true_value.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn interpret_recursive_unary_expression() {
        let plus_one = Parser::parse("--1").unwrap().interpret();
        let true_value = Parser::parse("!!true").unwrap().interpret();
        let false_value = Parser::parse("!!!true").unwrap().interpret();

        assert_eq!(plus_one.unwrap(), Value::Number(1f64));
        assert_eq!(true_value.unwrap(), Value::Boolean(true));
        assert_eq!(false_value.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn interpret_binary_expressions() {
        let addition = Parser::parse("1 + 3").unwrap().interpret();
        let substraction = Parser::parse("1 - 3").unwrap().interpret();
        let multiplication = Parser::parse("2 * 3").unwrap().interpret();
        let division = Parser::parse("2 / 3").unwrap().interpret();

        assert_eq!(addition.unwrap(), Value::Number(4f64));
        assert_eq!(substraction.unwrap(), Value::Number(1.0 - 3.0));
        assert_eq!(multiplication.unwrap(), Value::Number(2.0 * 3.0));
        assert_eq!(division.unwrap(), Value::Number(2.0 / 3.0));
    }

    #[test]
    fn interpret_complex_expresion() {
        let arithmetic_combination = Parser::parse("1 + 2 * 3 / (23 + 43)").unwrap().interpret();
        let comparisons = Parser::parse("1 + 2 * 3 / (23 + 43) <= 123 == false")
            .unwrap()
            .interpret();

        let result = 1f64 + 2f64 * 3f64 / (23f64 + 43f64);
        assert_eq!(arithmetic_combination.unwrap(), Value::Number(result));
        assert_eq!(
            comparisons.unwrap(),
            Value::Boolean((result < 123f64) == false)
        );
    }

    #[test]
    fn interpret_ternary_expression() {
        let actual1 = Parser::parse("true ? 1, 2 : 3").unwrap().interpret();
        let actual2 = Parser::parse("true ? 1 : 2, 3").unwrap().interpret();
        let actual3 = Parser::parse("false ? 2, 3: 1").unwrap().interpret();

        assert_eq!(actual1.unwrap(), Value::Number(2f64));
        assert_eq!(actual2.unwrap(), Value::Number(3f64));
        assert_eq!(actual3.unwrap(), Value::Number(1f64));
    }
}
