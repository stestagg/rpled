use crate::ast::Spanned;
use chumsky::input::InputRef;
use chumsky::prelude::*;
use chumsky::extension::v1::{ExtParser, Ext};


#[derive(Clone, Debug, PartialEq)]
pub enum Constant{
    Num(i16),
    Float(f32),
    String(String),
    True,
    False,
    Nil,
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Num(n) => write!(f, "{}", n),
            Constant::Float(x) => write!(f, "{}", x),
            Constant::String(s) => write!(f, "{}", s),
            Constant::True => write!(f, "true"),
            Constant::False => write!(f, "false"),
            Constant::Nil => write!(f, "nil"),
        }
    }
}

crate::parser! {
    ConstantParser(inp) -> Result<Constant> {
        let number = text::int(10)
            .to_slice()
            .from_str()
            .unwrapped()
            .map(Constant::Num);

        let exponent = one_of("eE")
            .then(one_of("+-").or_not())
            .then(text::int(10))
            .map(|((e, sign), digits)| {
                let exp: i32 = digits.parse().unwrap();
                if let Some(s) = sign {
                    if s == '-' {
                        -exp
                    } else {
                        exp
                    }
                } else {
                    exp
                }
            });

        let float = text::int(10)
            .then(just('.').then(text::digits(10)).or_not())
            .then(exponent.or_not())
            .to_slice()
            .from_str()
            .unwrapped()
            .map(Constant::Float);


        let escape = just('\\').then(choice((
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('"').to('"'),
            just('\'').to('\''),
            just('\\').to('\\'),
            just('0').to('\0'),
        )))
        .ignored();

        let double_string = none_of("\"")
            .ignored()
            .or(escape)
            .repeated()
            .to_slice()
            .map(Constant::String)
            .delimited_by(just('"'), just('"'));

        let single_string = none_of("'")
            .ignored()
            .or(escape)
            .repeated()
            .to_slice()
            .map(Constant::String)
            .delimited_by(just('\''), just('\''));

        let string = double_string.or(single_string);

        let boolean = choice((
            just("true").to(Constant::True),
            just("false").to(Constant::False),
        ));

        let nil = just("nil").to(Constant::Nil);

        let constant = choice((
            float,
            number,
            string,
            boolean,
            nil,
        ));
        constant.parse(inp)
    }
}