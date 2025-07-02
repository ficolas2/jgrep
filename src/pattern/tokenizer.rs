use crate::errors::parsing_error::ParsingError;

#[derive(Debug, PartialEq, Eq)]
pub struct TextContent {
    pub str: String,
    pub quoted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(TextContent),
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

impl Token {
    pub fn get_original_string(&self) -> String {
        match self {
            Token::Text(TextContent { str, quoted: true }) => format!("\"{str}\""),
            Token::Text(TextContent { str, quoted: false }) => str.into(),
            Token::Dot => ".".into(),
            Token::Colon => ":".into(),
            Token::Dollar => "$".into(),
            Token::BracketExpr(tokens) => {
                let mut result = String::new();
                result.push('[');
                for token in tokens {
                    result.push_str(&token.get_original_string());
                }
                result.push(']');
                result
            },
        }
    }
}

impl BracketToken {
    pub fn get_original_string(&self) -> String {
        match self {
            BracketToken::Text(TextContent { str, quoted: true }) => format!("\"{str}\""),
            BracketToken::Text(TextContent { str, quoted: false }) => str.into(),
            BracketToken::Int(i) => format!("{}", i),
            BracketToken::Colon => ":".into(),
            BracketToken::Comma => ",".into(),
        }
    }
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
            ':' => {
                i += 1;
                Token::Colon
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

fn parse_quoted(chars: &Vec<char>, start_i: &mut usize) -> Result<TextContent, ParsingError> {
    *start_i += 1;
    let mut i = *start_i;

    while i < chars.len() {
        if chars[i] == '"' {
            let str = chars[*start_i..i].iter().collect();
            *start_i = i + 1;
            return Ok(TextContent{
                str,
                quoted: true,
            });
        }
        i += 1;
    }

    return Err(ParsingError::missmatched_quotes(*start_i - 1));
}

fn parse_text(
    chars: &Vec<char>,
    start_i: &mut usize,
    stop_chars: &[char],
) -> Result<TextContent, ParsingError> {
    let mut i = *start_i;

    while i < chars.len() && !stop_chars.contains(&chars[i]) {
        i += 1;
    }

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
        let chars = r#"ab."a.c"."#.chars().collect();

        let mut i = 3;

        let res = parse_quoted(&chars, &mut i);

        assert_eq!(TextContent{str: "a.c".to_string(), quoted: true}, res.unwrap());
        assert_eq!(8, i);
    }

    #[test]
    fn test_parse_quoted_missmatched() {
        let chars = r#"ab."a.c."#.chars().collect();
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
                Token::Text(TextContent{str: "val".to_string(), quoted: false}),
            ],
            res.unwrap(),
        );
    }
}
