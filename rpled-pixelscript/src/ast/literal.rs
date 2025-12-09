use crate::ast::Spanned;
use std::fmt;
use chumsky::{prelude::*, primitive};
use chumsky::extension::v1::{ExtParser, Ext};


#[derive(Clone, Debug, PartialEq, Display)]
pub enum Literal{
    Num(i16),
    Float(f32),
    String(String),
    True,
    False,
    Nil,
}

impl<'src, I, E> ExtParser<'src, I, Spanned<Literal>, E> for Literal
where
    I: Input<'src, Token = u8>,
    E: Err<Rich<'src, char, Span>>,
{
    fn parse(&self, inp: &mut InputRef<'src, '_, I, E>) -> Result<Spanned<Literal>, E> {
        let number = text::int(10)
            .to_slice()
            .from_str()
            .unwrapped()
            .map(Literal::Num);

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
            .map(Literal::Float);


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
            .map(Literal::String)
            .delimited_by(just('"'), just('"'));

        let single_string = none_of("'")
            .ignored()
            .or(escape)
            .repeated()
            .to_slice()
            .map(Literal::String)
            .delimited_by(just('\''), just('\''));

        let string = double_string.or(single_string);

        let boolean = choice((
            just("true").to(Literal::True),
            just("false").to(Literal::False),
        ));

        let nil = just("nil").to(Literal::Nil);

        let literal = choice((
            float,
            number,
            string,
            boolean,
            nil,
        ));
        literal.parse(inp)
    }
}