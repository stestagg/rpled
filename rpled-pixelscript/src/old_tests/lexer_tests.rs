use crate::lexer::{lex, Token};

#[test]
fn test_lex_keywords() {
    let result = lex("if then else end").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::If, &Token::Then, &Token::Else, &Token::End]
    );
}

#[test]
fn test_lex_all_keywords() {
    let result = lex("and break do else elseif end false for function if in local nil not or repeat return then true until while").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(tokens.len(), 21);  // 21 keywords total
}

#[test]
fn test_lex_operators() {
    let result = lex("+ - * / % ^ == ~= < <= > >=").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Plus,
            &Token::Minus,
            &Token::Star,
            &Token::Slash,
            &Token::Percent,
            &Token::Caret,
            &Token::Eq,
            &Token::Ne,
            &Token::Lt,
            &Token::Le,
            &Token::Gt,
            &Token::Ge,
        ]
    );
}

#[test]
fn test_lex_delimiters() {
    let result = lex("( ) { } [ ] , ;").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::LParen,
            &Token::RParen,
            &Token::LBrace,
            &Token::RBrace,
            &Token::LBracket,
            &Token::RBracket,
            &Token::Comma,
            &Token::Semicolon,
        ]
    );
}

#[test]
fn test_lex_identifiers() {
    let result = lex("foo bar _baz test123 _123").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Ident("foo".to_string()),
            &Token::Ident("bar".to_string()),
            &Token::Ident("_baz".to_string()),
            &Token::Ident("test123".to_string()),
            &Token::Ident("_123".to_string()),
        ]
    );
}

#[test]
fn test_lex_numbers() {
    let result = lex("0 42 123 999").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::Number(0), &Token::Number(42), &Token::Number(123), &Token::Number(999)]
    );
}

#[test]
fn test_lex_floats() {
    let result = lex("3.14 0.5 123.456").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::Float(3.14), &Token::Float(0.5), &Token::Float(123.456)]
    );
}

#[test]
fn test_lex_floats_with_exponent() {
    let result = lex("1e10 1.5e-3 2E+5").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::Float(1e10), &Token::Float(1.5e-3), &Token::Float(2E+5)]
    );
}

#[test]
fn test_lex_string_double_quotes() {
    let result = lex(r#""hello world""#).unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(tokens, vec![&Token::String("hello world".to_string())]);
}

#[test]
fn test_lex_string_single_quotes() {
    let result = lex("'hello world'").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(tokens, vec![&Token::String("hello world".to_string())]);
}

#[test]
fn test_lex_string_escapes() {
    let result = lex(r#""hello\nworld\t\"test\"""#).unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::String("hello\nworld\t\"test\"".to_string())]
    );
}

#[test]
fn test_lex_comments() {
    let result = lex("foo -- this is a comment\nbar").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::Ident("foo".to_string()), &Token::Ident("bar".to_string())]
    );
}

#[test]
fn test_lex_assignment() {
    let result = lex("x = 5").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Ident("x".to_string()),
            &Token::Assign,
            &Token::Number(5)
        ]
    );
}

#[test]
fn test_lex_metadata_header() {
    let result = lex(r#"pixelscript = { name = "Test" }"#).unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Ident("pixelscript".to_string()),
            &Token::Assign,
            &Token::LBrace,
            &Token::Ident("name".to_string()),
            &Token::Assign,
            &Token::String("Test".to_string()),
            &Token::RBrace,
        ]
    );
}

#[test]
fn test_lex_function_def() {
    let result = lex("function foo(x, y) end").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Function,
            &Token::Ident("foo".to_string()),
            &Token::LParen,
            &Token::Ident("x".to_string()),
            &Token::Comma,
            &Token::Ident("y".to_string()),
            &Token::RParen,
            &Token::End,
        ]
    );
}

#[test]
fn test_lex_whitespace_handling() {
    let result = lex("  foo  \t  bar  \n  baz  ").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Ident("foo".to_string()),
            &Token::Ident("bar".to_string()),
            &Token::Ident("baz".to_string()),
        ]
    );
}

#[test]
fn test_lex_span_tracking() {
    let result = lex("foo bar").unwrap();
    assert_eq!(result[0].span, 0..3);  // "foo"
    assert_eq!(result[1].span, 4..7);  // "bar"
}

#[test]
fn test_lex_error_unterminated_string() {
    let result = lex(r#""hello"#);
    assert!(result.is_err());
}

#[test]
fn test_lex_error_invalid_escape() {
    let result = lex(r#""\x""#);
    assert!(result.is_err());
}

#[test]
fn test_lex_error_unexpected_char() {
    let result = lex("@");
    assert!(result.is_err());
}

#[test]
fn test_lex_qualified_names() {
    let result = lex("foo.bar").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![&Token::Ident("foo".to_string()), &Token::Dot, &Token::Ident("bar".to_string())]
    );
}

#[test]
fn test_lex_qualified_names_multi_level() {
    let result = lex("a.b.c").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(
        tokens,
        vec![
            &Token::Ident("a".to_string()),
            &Token::Dot,
            &Token::Ident("b".to_string()),
            &Token::Dot,
            &Token::Ident("c".to_string())
        ]
    );
}

#[test]
fn test_lex_dot_vs_float() {
    // Test that 123.456 is a float
    let result = lex("123.456").unwrap();
    let tokens: Vec<_> = result.iter().map(|t| &t.node).collect();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Float(_)));

    // Test that foo.bar has a dot token
    let result2 = lex("foo.bar").unwrap();
    let tokens2: Vec<_> = result2.iter().map(|t| &t.node).collect();
    assert_eq!(tokens2[1], &Token::Dot);

    // Test that 123. followed by identifier is number then dot then identifier
    let result3 = lex("123.foo").unwrap();
    let tokens3: Vec<_> = result3.iter().map(|t| &t.node).collect();
    assert_eq!(tokens3.len(), 3);
    assert!(matches!(tokens3[0], Token::Number(123)));
    assert_eq!(tokens3[1], &Token::Dot);
    assert_eq!(tokens3[2], &Token::Ident("foo".to_string()));
}
