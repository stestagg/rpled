use super::prelude::*;


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

parser!(for: Constant {
    let digits = text::digits(10).to_slice();
    let frac = just('.').then(digits);
    let exp = just('e')
        .or(just('E'))
        .then(one_of("+-").or_not())
        .then(digits);

    let float = just('-').or_not()
        .then(text::int(10))
        .then(frac.or_not())
        .then(exp.or_not())
        .to_slice()
        .map(|s: &str| Constant::Float(s.parse().unwrap()));
        
    let number = text::int(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Constant::Num);

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
        .map(|s: &str| Constant::String(s.to_string()))
        .delimited_by(just('"'), just('"'));

    let single_string = none_of("'")
        .ignored()
        .or(escape)
        .repeated()
        .to_slice()
        .map(|s: &str| Constant::String(s.to_string()))
        .delimited_by(just('\''), just('\''));

    let string = double_string.or(single_string);

    let boolean = choice((
        just("true").to(Constant::True),
        just("false").to(Constant::False),
    ));

    let nil = just("nil").to(Constant::Nil);

    choice((
        float,
        number,
        string,
        boolean,
        nil,
    ))
});