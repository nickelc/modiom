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

#[derive(Debug, Eq, PartialEq)]
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
                    }).collect::<Vec<String>>();
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
                let mut it = list.into_iter().peekable();
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

mod parser {
    use super::*;

    use nom::types::CompleteStr;
    use nom::{digit, eol, is_alphanumeric, multispace, non_empty};
    use nom::{Context, Err as NomError};

    named!(opt_multispace<CompleteStr<'_>, Option<CompleteStr<'_>>>,
        opt!(multispace)
    );

    named!(identifier<CompleteStr<'_>, String>,
        do_parse!(
            ident: take_while!(|c| is_alphanumeric(c as u8) || c == '_') >>
            (String::from(ident.0))
        )
    );

    named!(operator<CompleteStr<'_>, Operator>,
        alt!(
            value!(Operator::Equals, tag!("=")) |
            value!(Operator::NotEquals, tag!("!="))
        )
    );

    named!(operator_int<CompleteStr<'_>, Operator>,
        alt!(
            value!(Operator::Min, tag!(">=")) |
            value!(Operator::Max, tag!("<=")) |
            value!(Operator::SmallerThan, tag!("<")) |
            value!(Operator::GreaterThan, tag!(">")) |
            value!(Operator::BitwiseAnd, tag!("&"))
        )
    );

    named!(operator_str<CompleteStr<'_>, Operator>,
        alt!(
            value!(Operator::Like, tag_no_case!("like")) |
            value!(Operator::NotLike, tag_no_case!("not like"))
        )
    );

    named!(operator_lst<CompleteStr<'_>, Operator>,
        alt!(
            value!(Operator::In, tag_no_case!("in")) |
            value!(Operator::NotIn, tag_no_case!("not in"))
        )
    );

    named!(integer_literal<CompleteStr<'_>, Literal>,
        do_parse!(
            sign: opt!(tag!("-")) >>
            val: digit >>
            ({
                let mut intval = i64::from_str_radix(val.0, 10).unwrap();
                if sign.is_some() {
                    intval *= -1;
                }
                Literal::Integer(intval)
            })
        )
    );

    named!(string_literal<CompleteStr<'_>, Literal>,
        do_parse!(
            val: alt_complete!(
                delimited!(tag!("\""), opt!(take_until!("\"")), tag!("\""))
                | delimited!(tag!("'"), opt!(take_until!("'")), tag!("'"))
            ) >>
            ({
                let val = val.unwrap_or(CompleteStr(""));
                let s = String::from(val.0);
                Literal::String(s)
            })
        )
    );

    named!(literal<CompleteStr<'_>, Literal>,
        alt!(
            string_literal
            | integer_literal
        )
    );

    named!(value_list<CompleteStr<'_>, Vec<Literal>>,
        many0!(
            do_parse!(
                val: literal >>
                opt!(
                    do_parse!(
                        opt_multispace >>
                        tag!(",") >>
                        opt_multispace >>
                        ()
                    )
                ) >>
                (val)
            )
        )
    );

    named!(full_expr<CompleteStr<'_>, Expr>,
        do_parse!(
            left: identifier >>
            opt_multispace >>
            op_right: alt!(
                do_parse!(
                    op: operator >>
                    opt_multispace >>
                    right: literal >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_int >>
                    opt_multispace >>
                    right: integer_literal >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_str >>
                    opt_multispace >>
                    right: string_literal >>
                    ((op, Condition::Literal(right)))
                ) |
                do_parse!(
                    op: operator_lst >>
                    opt_multispace >>
                    right: delimited!(tag!("("), value_list, tag!(")")) >>
                    ((op, Condition::LiteralList(right)))
                )
            ) >>
            alt!(eof!() | eol) >>
            (Expr {
                property: left,
                op: op_right.0,
                right: op_right.1
            })
        )
    );

    named!(op_right_only<CompleteStr<'_>, (Option<Operator>, Condition)>,
        do_parse!(
            opt_multispace >>
            op_right: alt!(
                do_parse!(
                    op: opt!(operator) >>
                    opt_multispace >>
                    right: literal >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_int) >>
                    opt_multispace >>
                    right: integer_literal >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_str) >>
                    opt_multispace >>
                    right: string_literal >>
                    (op, Condition::Literal(right))
                ) |
                do_parse!(
                    op: opt!(operator_lst) >>
                    opt_multispace >>
                    right: delimited!(tag!("("), value_list, tag!(")")) >>
                    (op, Condition::LiteralList(right))
                ) |
                do_parse!(
                    op: opt!(alt!(
                        do_parse!(op: operator >> opt_multispace >> (op)) |
                        do_parse!(op: operator_str >> multispace >> (op))
                    )) >>
                    val: non_empty >>
                    ({
                        let s = String::from(val.0);
                        (op, Condition::Literal(Literal::String(s)))
                    })
                )
            ) >>
            alt!(eof!() | eol) >>
            (op_right)
        )
    );

    pub fn parse(expr: &str) -> Result<Expr, String> {
        match full_expr(CompleteStr(expr)) {
            Ok((_, e)) => Ok(e),
            Err(e) => {
                let msg = match e {
                    NomError::Error(Context::Code(c, _))
                    | NomError::Failure(Context::Code(c, _)) => {
                        format!("failed to parse {:?}", c.0)
                    }
                    NomError::Incomplete(_) => String::from("failed to parse expression"),
                };
                Err(msg)
            }
        }
    }

    pub fn parse_for(prop: &str, expr: &str) -> Result<Expr, String> {
        let (op, right) = match op_right_only(CompleteStr(expr)) {
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
                    NomError::Error(Context::Code(c, _))
                    | NomError::Failure(Context::Code(c, _)) => {
                        format!("failed to parse {:?}", c.0)
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
        }

        #[test]
        fn test_parse_for() {
            assert_eq!(
                parse_for("id", ""),
                Err(String::from("failed to parse \"\"")),
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
