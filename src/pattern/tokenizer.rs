use crate::errors::parsing_error::ParsingError;

#[derive(Debug, PartialEq, Eq)]
pub struct TextContent {
    pub str: String,
    pub quoted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(TextContent),
    Value(TextContent),
    BracketExpr(Vec<BracketToken>),
    Dot,
    Colon,
    Dollar,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BracketToken {
    Text(TextContent),
    Int(i32),
    Colon,
    Comma,
}

/// Position of the first `target` that a backslash does not escape.
///
/// Escapes are left in the string rather than resolved here, and wildcard_match is what turns
/// `\x` back into a literal x.
fn find_unescaped(chars: &[char], target: char) -> Option<usize> {
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            c if c == target => return Some(i),
            _ => i += 1,
        }
    }

    None
}

pub fn tokenize(pattern_str: &str) -> Result<Vec<Token>, ParsingError> {
    const TEXT_STOPS: &[char] = &['.', ':', '['];
    let mut tokens = Vec::new();
    let chars: Vec<char> = pattern_str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let token = match c {
            '.' => {
                i += 1;
                Token::Dot
            }
            // Everything after the colon is the value. It is not path syntax, so it is kept
            // whole instead of being split on `.`, `[` or `"`.
            ':' => {
                i += 1;
                tokens.push(Token::Colon);
                if let Some(value) = tokenize_value(&chars[i..], i)? {
                    // Bracket colons were consumed with their brackets and quoted ones are part
                    // of the value, so a colon still left here is a second top level one.
                    let value_chars: Vec<char> = value.str.chars().collect();
                    if !value.quoted && find_unescaped(&value_chars, ':').is_some() {
                        return Err(ParsingError::multiple_colons());
                    }
                    tokens.push(Token::Value(value));
                }
                break;
            }
            '$' => {
                i += 1;
                Token::Dollar
            }
            '[' => {
                let bracket_tokens = tokenize_inside_bracket(&chars, &mut i)?;
                Token::BracketExpr(bracket_tokens)
            }
            '"' => Token::Text(parse_quoted(&chars, &mut i)?),
            _ => Token::Text(parse_text(&chars, &mut i, TEXT_STOPS)?),
        };

        tokens.push(token)
    }

    Ok(tokens)
}

pub fn tokenize_value(chars: &[char], offset: usize) -> Result<Option<TextContent>, ParsingError> {
    let Some(start) = chars.iter().position(|c| !c.is_whitespace()) else {
        return Ok(None);
    };
    let end = chars.iter().rposition(|c| !c.is_whitespace()).unwrap() + 1;
    let trimmed = &chars[start..end];

    // Empty case
    if trimmed[0] != '"' {
        return Ok(Some(TextContent {
            str: trimmed.iter().collect(),
            quoted: false,
        }));
    }

    // Once a value opens with a quote, the last character has to close it, with no unescaped
    // quotes in between.
    let inner = &trimmed[1..];
    if find_unescaped(inner, '"') != Some(inner.len().wrapping_sub(1)) {
        return Err(ParsingError::missmatched_quotes(offset + start));
    }
    let inner = &inner[..inner.len() - 1];

    Ok(Some(TextContent {
        str: inner.iter().collect(),
        quoted: true,
    }))
}

fn tokenize_inside_bracket(
    chars: &Vec<char>,
    start_i: &mut usize,
) -> Result<Vec<BracketToken>, ParsingError> {
    let mut tokens = Vec::new();
    const TEXT_STOPS: &[char] = &[']'];
    let mut i = *start_i + 1;

    while i < chars.len() {
        let c = chars[i];

        #[rustfmt::skip]
        let token = match c {
            ']' => {
                *start_i = i + 1;
                return Ok(tokens);
            }
            ',' => { i+=1; BracketToken::Comma },
            ':' => { i+=1; BracketToken::Colon },
            c if c.is_ascii_digit() => BracketToken::Int(parse_int(&chars, &mut i)?),
            '-' => {
                i += 1;
                BracketToken::Int(-parse_int(&chars, &mut i)?)
            }
            '"' => BracketToken::Text(parse_quoted(&chars, &mut i)?),
            _ => BracketToken::Text(parse_text(&chars, &mut i, &TEXT_STOPS)?),
        };

        tokens.push(token);
    }

    return Err(ParsingError::missmatched_brackets(*start_i));
}

fn parse_quoted(chars: &[char], start_i: &mut usize) -> Result<TextContent, ParsingError> {
    *start_i += 1;

    let Some(close) = find_unescaped(&chars[*start_i..], '"') else {
        return Err(ParsingError::missmatched_quotes(*start_i - 1));
    };
    let close = *start_i + close;

    let str = chars[*start_i..close].iter().collect();
    *start_i = close + 1;

    Ok(TextContent { str, quoted: true })
}

fn parse_text(
    chars: &Vec<char>,
    start_i: &mut usize,
    stop_chars: &[char],
) -> Result<TextContent, ParsingError> {
    let mut i = *start_i;

    while i < chars.len() && !stop_chars.contains(&chars[i]) {
        i += if chars[i] == '\\' { 2 } else { 1 };
    }
    let i = i.min(chars.len());

    let str: String = chars[*start_i..i].iter().collect();
    *start_i = i;
    Ok(TextContent { str, quoted: false })
}

fn parse_int(chars: &Vec<char>, start_i: &mut usize) -> Result<i32, ParsingError> {
    let mut i = *start_i;

    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }

    let number_str: String = chars[*start_i..i].iter().collect();
    *start_i = i;

    number_str
        .parse::<i32>()
        .map_err(|_| ParsingError::integer_is_too_big(&number_str))
}

#[cfg(test)]
mod test {
    use crate::{errors::parsing_error::ParsingError, pattern::tokenizer::{parse_quoted, BracketToken, TextContent, Token}};

    use super::{parse_text, parse_int, tokenize, tokenize_inside_bracket};

    #[test]
    fn test_parse_quoted() {
        let chars: Vec<char> = r#"ab."a.c"."#.chars().collect();

        let mut i = 3;

        let res = parse_quoted(&chars, &mut i);

        assert_eq!(TextContent{str: "a.c".to_string(), quoted: true}, res.unwrap());
        assert_eq!(8, i);
    }

    #[test]
    fn test_parse_quoted_missmatched() {
        let chars: Vec<char> = r#"ab."a.c."#.chars().collect();
        let mut i = 3;
        let res = parse_quoted(&chars, &mut i);

        assert_eq!(res, Err(ParsingError::missmatched_quotes(3)));
    }

    #[test]
    fn test_parse_text() {
        const TEXT_STOPS: &[char] = &['.'];
        let chars = "ab.abc.".chars().collect();

        let mut i = 3;

        let res = parse_text(&chars, &mut i, &TEXT_STOPS);

        assert_eq!(TextContent{str: "abc".to_string(), quoted: false}, res.unwrap());
        assert_eq!(6, i);
    }

    #[test]
    fn test_parse_int() {
        let chars = "a12a".chars().collect();
        let mut i = 1;
        let res = parse_int(&chars, &mut i);

        assert_eq!(12, res.unwrap());
        assert_eq!(3, i);
    }

    #[test]
    fn test_parse_int_too_big() {
        let number = "1212431245125213614616341612";
        let chars = format!("a{}ww", number).chars().collect();
        let mut i = 1;
        let res = parse_int(&chars, &mut i);

        assert_eq!(res, Err(ParsingError::integer_is_too_big(number)));
    }

    #[test]
    fn test_tokenize_inside_bracket() {
        let chars = r#"a[1,-2,"quoted":word]"#.chars().collect();
        let mut i = 1;
        let res = tokenize_inside_bracket(&chars, &mut i);

        assert_eq!(
            vec![
                BracketToken::Int(1),
                BracketToken::Comma,
                BracketToken::Int(-2),
                BracketToken::Comma,
                BracketToken::Text(TextContent{str: "quoted".to_string(), quoted: true}),
                BracketToken::Colon,
                BracketToken::Text(TextContent{str: "word".to_string(), quoted: false}),
            ],
            res.unwrap(),
        );
        assert_eq!(21, i);
    }

    #[test]
    fn test_tokenize_inside_bracket_missmatched() {
        let chars = r#"a[1,"quoted""#.chars().collect();
        let mut i = 1;
        let res = tokenize_inside_bracket(&chars, &mut i);

        assert_eq!(res, Err(ParsingError::missmatched_brackets(1)));
    }

    #[test]
    fn test_tokenize() {
        let str = r#".field."quotedfield".[1,23,-3][1:2]:val"#;
        let res = tokenize(str);

        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: "field".to_string(), quoted: false}),
                Token::Dot,
                Token::Text(TextContent{str: "quotedfield".to_string(), quoted: true}),
                Token::Dot,
                Token::BracketExpr(vec![
                    BracketToken::Int(1),
                    BracketToken::Comma,
                    BracketToken::Int(23),
                    BracketToken::Comma,
                    BracketToken::Int(-3),
                ]),
                Token::BracketExpr(vec![
                    BracketToken::Int(1),
                    BracketToken::Colon,
                    BracketToken::Int(2)
                ]),
                Token::Colon,
                Token::Value(TextContent{str: "val".to_string(), quoted: false}),
            ],
            res.unwrap(),
        );
    }

    #[test]
    fn test_tokenize_value() {
        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: "date".to_string(), quoted: false}),
                Token::Colon,
                Token::Value(TextContent{str: "03/05/2026".to_string(), quoted: false}),
            ],
            tokenize(".date: 03/05/2026").unwrap(),
        );
    }

    // A colon stays syntax, and has to be quoted to be part of a value
    #[test]
    fn test_tokenize_value_colon() {
        assert_eq!(Err(ParsingError::multiple_colons()), tokenize(".time: 10:30"));
        assert_eq!(Err(ParsingError::multiple_colons()), tokenize("a: b: c"));

        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: "time".to_string(), quoted: false}),
                Token::Colon,
                Token::Value(TextContent{str: "10:30".to_string(), quoted: true}),
            ],
            tokenize(r#".time: "10:30""#).unwrap(),
        );
    }

    // Range colons are consumed with the brackets, so they never reach the value
    #[test]
    fn test_tokenize_value_after_range() {
        assert_eq!(
            vec![
                Token::BracketExpr(vec![
                    BracketToken::Int(1),
                    BracketToken::Colon,
                    BracketToken::Int(2),
                ]),
                Token::Colon,
                Token::Value(TextContent{str: "x".to_string(), quoted: false}),
            ],
            tokenize("[1:2]: x").unwrap(),
        );
    }

    // The value is not path syntax, so it keeps its own dots and brackets
    #[test]
    fn test_tokenize_value_quoted() {
        assert_eq!(
            vec![
                Token::Colon,
                Token::Value(TextContent{str: "a.b[0]".to_string(), quoted: true}),
            ],
            tokenize(r#": "a.b[0]""#).unwrap(),
        );
    }

    #[test]
    fn test_tokenize_value_empty() {
        assert_eq!(
            vec![
                Token::Text(TextContent{str: "date".to_string(), quoted: false}),
                Token::Colon,
            ],
            tokenize("date:  ").unwrap(),
        );
    }

    #[test]
    fn test_tokenize_value_missmatched_quotes() {
        assert_eq!(Err(ParsingError::missmatched_quotes(2)), tokenize(r#": "abc"#));
        assert_eq!(Err(ParsingError::missmatched_quotes(1)), tokenize(r#":"a""b""#));
    }

    // A backslash keeps the next character from being read as syntax. The escape itself stays in
    // the string, and wildcard_match is what resolves it.
    #[test]
    fn test_tokenize_escaped_quote() {
        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: r#"a\"b"#.to_string(), quoted: true}),
            ],
            tokenize(r#"."a\"b""#).unwrap(),
        );
    }

    #[test]
    fn test_tokenize_escaped_stops() {
        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: r"a\.b\[0\]".to_string(), quoted: false}),
            ],
            tokenize(r".a\.b\[0\]").unwrap(),
        );
    }

    // An escaped colon is not the one that starts the value, nor a second top level one
    #[test]
    fn test_tokenize_escaped_colon() {
        assert_eq!(
            vec![
                Token::Text(TextContent{str: r"a\:b".to_string(), quoted: false}),
            ],
            tokenize(r"a\:b").unwrap(),
        );

        assert_eq!(
            vec![
                Token::Dot,
                Token::Text(TextContent{str: "time".to_string(), quoted: false}),
                Token::Colon,
                Token::Value(TextContent{str: r"10\:30".to_string(), quoted: false}),
            ],
            tokenize(r".time: 10\:30").unwrap(),
        );
    }

    #[test]
    fn test_tokenize_value_escaped_quote() {
        assert_eq!(
            vec![
                Token::Colon,
                Token::Value(TextContent{str: r#"a\"b"#.to_string(), quoted: true}),
            ],
            tokenize(r#": "a\"b""#).unwrap(),
        );
    }

    // An unterminated quote is still an error, the backslash does not close it
    #[test]
    fn test_tokenize_escaped_quote_missmatched() {
        assert_eq!(Err(ParsingError::missmatched_quotes(1)), tokenize(r#"."a\"b"#));
    }
}
