use crate::parser_ext::InlinePadExt;
use crate::ast::Extra;
use chumsky::prelude::*;

#[test]
fn test_inlinepad_spaces() {
    let parser = just::<_, _,  Extra>("hello").inlinepad();

    // Should work with spaces
    assert!(parser.parse("  hello  ").into_result().is_ok());
    assert!(parser.parse("hello").into_result().is_ok());
    assert!(parser.parse("  hello").into_result().is_ok());
    assert!(parser.parse("hello  ").into_result().is_ok());
}

#[test]
fn test_inlinepad_tabs() {
    let parser = just::<_, _, Extra>("hello").inlinepad();

    // Should work with tabs
    assert!(parser.parse("\t\thello\t\t").into_result().is_ok());
    assert!(parser.parse("\thello").into_result().is_ok());
    assert!(parser.parse("hello\t").into_result().is_ok());
}

#[test]
fn test_inlinepad_mixed_whitespace() {
    let parser = just::<_, _, Extra>("hello").inlinepad();

    // Should work with mixed spaces and tabs
    assert!(parser.parse(" \t hello \t ").into_result().is_ok());
}

#[test]
fn test_inlinepad_no_newlines() {
    let parser = just::<_, _, Extra>("hello").inlinepad();

    // Should NOT consume newlines
    assert!(parser.parse("\nhello").into_result().is_err());
    assert!(parser.parse("hello\n").into_result().is_err());
    assert!(parser.parse("\n\nhello\n\n").into_result().is_err());
}

#[test]
fn test_inlinepad_vs_padded() {
    let padded_parser = just::<_, _, Extra>("hello").padded();
    let inlinepad_parser = just::<_, _, Extra>("hello").inlinepad();

    // padded() should work with newlines
    assert!(padded_parser.parse("\nhello\n").into_result().is_ok());

    // inlinepad() should NOT work with newlines
    assert!(inlinepad_parser.parse("\nhello\n").into_result().is_err());

    // Both should work with spaces
    assert!(padded_parser.parse("  hello  ").into_result().is_ok());
    assert!(inlinepad_parser.parse("  hello  ").into_result().is_ok());
}
