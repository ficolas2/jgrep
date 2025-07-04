use crate::{
    errors::parsing_error::ParsingError,
    pattern::{
        pattern_node::IndexPattern,
        tokenizer::{TextContent, Token},
    },
};

use super::{
    pattern::Pattern, pattern_node::PatternNode, tokenizer::{tokenize, BracketToken}
};

pub fn parse(pattern_str: &str) -> Result<Pattern, ParsingError> {
    let mut tokens = tokenize(pattern_str.trim())?;

    // Add wildcards to extremes, if they are unquoted idents
    match &mut tokens.first_mut() {
        Some(Token::Text(tc)) if !tc.quoted => {
            tc.str = format!("*{}", tc.str);
        }
        _ => {}
    }

    match &mut tokens.last_mut() {
        Some(Token::Text(tc)) if !tc.quoted => {
            tc.str = format!("{}*", tc.str);
        }
        _ => {}
    }

    let colon_indices = token_indices(&tokens, Token::Colon);

    let (path, value, or, start_at_root) = match colon_indices.as_slice() {
        [] => {
            let (path, start_at_root) = parse_path(&tokens)?;
            let value = if start_at_root
                || matches!(tokens.first(), Some(Token::BracketExpr(_) | &Token::Dot))
            {
                None
            } else {
                parse_value(&tokens)?
            };
            let or = value.is_some();
            (path, value, or, start_at_root)
        }
        [i] => {
            let (path, start_at_root) = parse_path(&tokens[..*i])?;
            let value = parse_value(&tokens[i + 1..])?;
            (path, value, false, start_at_root)
        }
        _ => return Err(ParsingError::multiple_colons()),
    };

    Ok(Pattern {
        path,
        value,
        or,
        start_at_root,
    })
}

fn parse_path(tokens: &[Token]) -> Result<(Vec<PatternNode>, bool), ParsingError> {
    let mut start_at_root = false;
    let mut pattern_vec: Vec<PatternNode> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        let token = match token {
            Token::Text(text_content) => Some(PatternNode::Key(text_content.str.clone())),
            Token::BracketExpr(bracket_tokens) => Some(parse_brackets(bracket_tokens)?),
            Token::Dollar => {
                if i != 0 {
                    return Err(ParsingError::unexpected_dollar());
                }
                start_at_root = true;
                None
            }
            Token::Dot => {
                let prev = tokens.get(i.wrapping_sub(1));
                let prev2 = tokens.get(i.wrapping_sub(2));
                match (prev2, prev) {
                    (Some(&Token::Dot), Some(&Token::Dot)) => {
                        return Err(ParsingError::too_many_dots())
                    }
                    (_, Some(Token::Dot)) => Some(PatternNode::Recursive()),
                    (_, _) => None,
                }
            }
            Token::Colon => unreachable!(),
        };

        token.map(|t| pattern_vec.push(t));
    }

    Ok((pattern_vec, start_at_root))
}

fn parse_value(tokens: &[Token]) -> Result<Option<String>, ParsingError> {
    if tokens.is_empty() {
        return Ok(None)
    }
    // Any amount of spaces before and a quoted string
    match tokens {
        [Token::Text(TextContent {
            str, quoted: false, ..
        })] => return Ok(Some(str.trim().into())),

        [Token::Text(TextContent {
            str: spaces_str,
            quoted: false,
            ..
        }), Token::Text(TextContent {
            str, quoted: true, ..
        })] if spaces_str.chars().all(|c| c == ' ') => return Ok(Some(str.into())),
        _ => {}
    }

    // Other
    let mut result = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i != 0 {
            result.push_str(&token.get_original_string());
            continue;
        }
        if let Token::Text(TextContent { str, quoted: false }) = token {
            result.push_str(str.trim());
        } else {
            result.push_str(&token.get_original_string());
        }
    }

    Ok(Some(result))
}

fn parse_brackets(tokens: &[BracketToken]) -> Result<PatternNode, ParsingError> {
    let colon_indices = token_indices(tokens, BracketToken::Colon);
    let comma_indices = token_indices(tokens, BracketToken::Comma);

    // [] or []
    if tokens.is_empty()
        || matches!(
            tokens,
            [BracketToken::Text(TextContent { str, quoted })] if str == "*" && *quoted == false
        )
    {
        return Ok(PatternNode::Index(IndexPattern::All));
    }

    // [n]
    if let [BracketToken::Int(n)] = tokens {
        return usize::try_from(*n)
            .map(|n| PatternNode::Index(IndexPattern::List(vec![n])))
            .map_err(|_| ParsingError::int_negative(*n));
    }

    //["field"]
    if let [BracketToken::Text(TextContent { str, quoted: true })] = tokens {
        return Ok(PatternNode::Key(str.into()));
    }

    // Range + Last N
    match colon_indices.as_slice() {
        [] => {}
        [i] => {
            let left = &tokens[..*i];
            let right = &tokens[i + 1..];

            let index_pattern = match (left, right) {
                // [:]
                ([], []) => IndexPattern::All,
                // [l:r]
                ([BracketToken::Int(l)], [BracketToken::Int(r)]) => {
                    let l = usize::try_from(*l).map_err(|_| ParsingError::int_negative(*l))?;
                    let r = usize::try_from(*r).map_err(|_| ParsingError::int_negative(*r))?;
                    IndexPattern::Range(l, Some(r))
                }
                // [l:] and [-l:]
                ([BracketToken::Int(l)], []) => {
                    if *l < 0 {
                        IndexPattern::LastN((-l) as usize)
                    } else {
                        IndexPattern::Range(*l as usize, None)
                    }
                }
                //[:r]
                ([], [BracketToken::Int(r)]) => IndexPattern::Range(
                    0,
                    Some(usize::try_from(*r).map_err(|_| ParsingError::int_negative(*r))?),
                ),
                _ => return Err(ParsingError::malformed_range()),
            };

            return Ok(PatternNode::Index(index_pattern));
        }
        [..] => return Err(ParsingError::malformed_range()),
    }

    // List
    if !comma_indices.is_empty() {
        let mut expect_comma = false;
        let mut indexes: Vec<usize> = Vec::new();
        for token in tokens {
            match (expect_comma, token) {
                (true, BracketToken::Comma) => expect_comma = false,
                (false, BracketToken::Int(i)) => {
                    if *i < 0 {
                        return Err(ParsingError::malformed_list());
                    }
                    indexes.push(*i as usize);
                    expect_comma = true;
                }
                _ => return Err(ParsingError::malformed_list()),
            }
        }

        return Ok(PatternNode::Index(IndexPattern::List(indexes)));
    }

    Err(ParsingError::malformed_range())
}

fn token_indices<T: PartialEq>(tokens: &[T], comp: T) -> Vec<usize> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(i, t)| (t == &comp).then_some(i))
        .collect()
}
