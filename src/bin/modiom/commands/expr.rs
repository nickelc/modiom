use std::fmt;

use modio::filter::{OneOrMany, Operator as FilterOp};

pub use self::parser::{parse, parse_for};

#[derive(Debug, Eq, PartialEq)]
pub enum Literal {
    Integer(i64),
    String(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Condition {
    Literal(Literal),
    LiteralList(Vec<Literal>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    Like,
    NotLike,
    In,
    NotIn,
    Min,
    Max,
    GreaterThan,
    SmallerThan,
    BitwiseAnd,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Operator::Equals => write!(f, "="),
            Operator::NotEquals => write!(f, "!="),
            Operator::Like => write!(f, "like"),
            Operator::NotLike => write!(f, "not like"),
            Operator::In => write!(f, "in"),
            Operator::NotIn => write!(f, "not in"),
            Operator::Min => write!(f, ">="),
            Operator::Max => write!(f, "<="),
            Operator::GreaterThan => write!(f, ">"),
            Operator::SmallerThan => write!(f, "<"),
            Operator::BitwiseAnd => write!(f, "&"),
        }
    }
}

impl From<Operator> for FilterOp {
    fn from(op: Operator) -> FilterOp {
        match op {
            Operator::Equals => FilterOp::Equals,
            Operator::NotEquals => FilterOp::Not,
            Operator::Like => FilterOp::Like,
            Operator::NotLike => FilterOp::NotLike,
            Operator::In => FilterOp::In,
            Operator::NotIn => FilterOp::NotIn,
            Operator::Min => FilterOp::Min,
            Operator::Max => FilterOp::Max,
            Operator::GreaterThan => FilterOp::GreaterThan,
            Operator::SmallerThan => FilterOp::SmallerThan,
            Operator::BitwiseAnd => FilterOp::BitwiseAnd,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Expr {
    pub property: String,
    pub op: Operator,
    pub right: Condition,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.property, self.op, self.right)
    }
}

impl Condition {
    pub fn into_value(self) -> OneOrMany<String> {
        match self {
            Condition::Literal(Literal::Integer(i)) => OneOrMany::One(i.to_string()),
            Condition::Literal(Literal::String(s)) => OneOrMany::One(s),
            Condition::LiteralList(list) => {
                let val = list
                    .into_iter()
                    .map(|l| match l {
                        Literal::Integer(i) => i.to_string(),
                        Literal::String(s) => s,
                    })
                    .collect::<Vec<String>>();
                OneOrMany::Many(val)
            }
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Literal(Literal::Integer(i)) => fmt::Display::fmt(i, f),
            Condition::Literal(Literal::String(s)) => fmt::Debug::fmt(s, f),
            Condition::LiteralList(list) => {
                write!(f, "(")?;
                let mut it = list.iter().peekable();
                while let Some(e) = it.next() {
                    match e {
                        Literal::Integer(i) => fmt::Display::fmt(i, f),
                        Literal::String(s) => fmt::Debug::fmt(s, f),
                    }?;
                    if it.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}

#[rustfmt::skip]
mod parser {
    use super::*;

    use nom::branch::alt;
    use nom::bytes::complete::{tag, tag_no_case, take_while};
    use nom::character::complete::{digit1, multispace0};
    use nom::character::is_alphanumeric;
    use nom::combinator::{rest, value};
    use nom::multi::many0;
    use nom::{alt, delimited, do_parse, opt, tag, take_until, verify};
    use nom::{Err as NomError, IResult, error::Error};

    #[inline]
    fn identifier(i: &str) -> IResult<&str, &str> {
        take_while(|c| is_alphanumeric(c as u8) || c == '_')(i)
    }

    #[inline]
    fn operator_eq(i: &str) -> IResult<&str, Operator> {
        let eq = value(Operator::Equals, tag("="));
        let not = value(Operator::NotEquals, tag("!="));

        alt((eq, not))(i)
    }

    #[inline]
    fn operator_int(i: &str) -> IResult<&str, Operator> {
        let min = value(Operator::Min, tag(">="));
        let max = value(Operator::Max, tag("<="));
        let lt = value(Operator::SmallerThan, tag("<"));
        let gt = value(Operator::GreaterThan, tag(">"));
        let bit = value(Operator::BitwiseAnd, tag("&"));

        alt((min, max, lt, gt, bit))(i)
    }

    #[inline]
    fn operator_str(i: &str) -> IResult<&str, Operator> {
        let like = value(Operator::Like, tag_no_case("like"));
        let not_like = value(Operator::NotLike, tag_no_case("not like"));

        alt((like, not_like))(i)
    }

    #[inline]
    fn operator_lst(i: &str) -> IResult<&str, Operator> {
        let _in = value(Operator::In, tag_no_case("in"));
        let not_in = value(Operator::NotIn, tag_no_case("not in"));

        alt((_in, not_in))(i)
    }

    #[inline]
    fn integer_lit(i: &str) -> IResult<&str, Literal> {
        do_parse!(
            i,
            sign: opt!(tag!("-")) >>
            val: digit1 >>
            ({
                let mut intval = i64::from_str_radix(val, 10).unwrap();
                if sign.is_some() {
                    intval *= -1;
                }
                Literal::Integer(intval)
            })
        )
    }

    #[inline]
    fn string_lit(i: &str) -> IResult<&str, Literal> {
        do_parse!(
            i,
            val: alt!(
                delimited!(tag!("\""), opt!(take_until!("\"")), tag!("\""))
                | delimited!(tag!("'"), opt!(take_until!("'")), tag!("'"))
            ) >>
            ({
                let s = val.map(From::from).unwrap_or_else(String::new);
                Literal::String(s)
            })
        )
    }

    #[inline]
    fn literal(i: &str) -> IResult<&str, Literal> {
        alt((string_lit, integer_lit))(i)
    }

    #[inline]
    fn value_list(i: &str) -> IResult<&str, Vec<Literal>> {
        fn _value(i: &str) -> IResult<&str, Literal> {
            do_parse!(
                i,
                val: literal >>
                opt!(
                    do_parse!(
                        opt!(multispace0) >>
                        tag!(",") >>
                        opt!(multispace0) >>
                        ()
                    )
                ) >>
                (val)
            )
        }
        many0(_value)(i)
    }

    #[inline]
    fn full_expr(i: &str) -> IResult<&str, Expr> {
        do_parse!(
            i,
            left: identifier >>
            opt!(multispace0) >>
            op_right: alt!(
                do_parse!(
                    op: operator_eq >>
                    opt!(multispace0) >>
                    right: literal >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_int >>
                    opt!(multispace0) >>
                    right: integer_lit >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_str >>
                    opt!(multispace0) >>
                    right: string_lit >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_lst >>
                    opt!(multispace0) >>
                    right: delimited!(tag!("("), value_list, tag!(")")) >>
                    ((op, Condition::LiteralList(right)))
                )
            ) >>
            (Expr {
                property: String::from(left),
                op: op_right.0,
                right: op_right.1
            })
        )
    }

    #[allow(clippy::cognitive_complexity)]
    #[inline]
    fn op_right_only(i: &str) -> IResult<&str, (Option<Operator>, Condition)> {
        do_parse!(
            i,
            opt!(multispace0) >>
            op_right: alt!(
                do_parse!(
                    op: opt!(operator_eq) >>
                    opt!(multispace0) >>
                    right: literal >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_int) >>
                    opt!(multispace0) >>
                    right: integer_lit >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_str) >>
                    opt!(multispace0) >>
                    right: string_lit >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_lst) >>
                    opt!(multispace0) >>
                    right: delimited!(tag!("("), value_list, tag!(")")) >>
                    (op, Condition::LiteralList(right))
                ) |
                do_parse!(
                    op: opt!(alt!(
                        do_parse!(op: operator_eq >> opt!(multispace0) >> (op)) |
                        do_parse!(op: operator_str >> multispace0 >> (op))
                    )) >>
                    val: verify!(rest, |s: &str| !s.is_empty()) >>
                    ({
                        let s = String::from(val);
                        (op, Condition::Literal(Literal::String(s)))
                    })
                )
            ) >>
            (op_right)
        )
    }

    pub fn parse(expr: &str) -> Result<Expr, String> {
        match full_expr(expr) {
            Ok((_, e)) => Ok(e),
            Err(e) => {
                let msg = match e {
                    NomError::Error(Error { input, .. }) | NomError::Failure(Error { input, .. }) => {
                        format!("failed to parse {:?}", input)
                    }
                    NomError::Incomplete(_) => String::from("failed to parse expression"),
                };
                Err(msg)
            }
        }
    }

    pub fn parse_for(prop: &str, expr: &str) -> Result<Expr, String> {
        let (op, right) = match op_right_only(expr) {
            Ok((_, (Some(op), right))) => (op, right),
            Ok((_, (None, right))) => {
                let op = match right {
                    Condition::Literal(Literal::String(ref s)) if s.contains('*') => Operator::Like,
                    Condition::LiteralList(_) => Operator::In,
                    _ => Operator::Equals,
                };
                (op, right)
            }
            Err(e) => {
                let msg = match e {
                    NomError::Error(Error { input, .. }) | NomError::Failure(Error { input, .. }) => {
                        format!("failed to parse {:?}", input)
                    }
                    NomError::Incomplete(_) => String::from("failed to parse expression"),
                };
                return Err(msg);
            }
        };

        Ok(Expr {
            property: String::from(prop),
            op,
            right,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse() {
            assert_eq!(
                parse("id = 1"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::Integer(1)),
                }),
            );
            assert_eq!(
                parse("id > 1"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::GreaterThan,
                    right: Condition::Literal(Literal::Integer(1)),
                }),
            );
            assert_eq!(
                parse("id > a"),
                Err(String::from("failed to parse \"> a\"")),
            );
        }

        #[test]
        fn test_parse_for() {
            assert_eq!(
                parse_for("id", ""),
                Err(String::from("failed to parse expression")),
            );
            assert_eq!(
                parse_for("id", "1"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::Integer(1)),
                }),
            );
            assert_eq!(
                parse_for("id", "=1"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::Integer(1)),
                }),
            );
            assert!(parse_for("id", "= 1").is_ok());
            assert_eq!(
                parse_for("id", ">1"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::GreaterThan,
                    right: Condition::Literal(Literal::Integer(1)),
                }),
            );
            assert_eq!(
                parse_for("id", "(1,2)"),
                Ok(Expr {
                    property: String::from("id"),
                    op: Operator::In,
                    right: Condition::LiteralList(vec![Literal::Integer(1), Literal::Integer(2)]),
                }),
            );

            assert_eq!(
                parse_for("name", "'foobar'"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::String(String::from("foobar"))),
                }),
            );
            assert_eq!(
                parse_for("name", "='foobar'"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::String(String::from("foobar"))),
                }),
            );
            assert_eq!(
                parse_for("name", "!='foobar'"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::NotEquals,
                    right: Condition::Literal(Literal::String(String::from("foobar"))),
                }),
            );
            assert_eq!(
                parse_for("name", "=foobar"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Equals,
                    right: Condition::Literal(Literal::String(String::from("foobar"))),
                }),
            );
            assert_eq!(
                parse_for("name", "!=foobar"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::NotEquals,
                    right: Condition::Literal(Literal::String(String::from("foobar"))),
                }),
            );
            assert_eq!(
                parse_for("name", "'foobar*'"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Like,
                    right: Condition::Literal(Literal::String(String::from("foobar*"))),
                }),
            );
            assert_eq!(
                parse_for("name", "foobar*"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Like,
                    right: Condition::Literal(Literal::String(String::from("foobar*"))),
                }),
            );
            assert_eq!(
                parse_for("name", "like \"*foobar*\""),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::Like,
                    right: Condition::Literal(Literal::String(String::from("*foobar*"))),
                }),
            );
            assert_eq!(
                parse_for("name", "not like *foobar*"),
                Ok(Expr {
                    property: String::from("name"),
                    op: Operator::NotLike,
                    right: Condition::Literal(Literal::String(String::from("*foobar*"))),
                }),
            );
            assert!(parse_for("name", "= ''").is_ok());
            assert!(parse_for("name", "!= ''").is_ok());
            assert!(parse_for("name", "= *foo*").is_ok());
            assert!(parse_for("name", "=").is_err());
        }
    }
}
