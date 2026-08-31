use crate::{
    errors::parsing_error::ParsingError,
    pattern::{
        pattern_node::IndexPattern,
        tokenizer::{TextContent, Token},
    },
};

use super::{
    pattern::Pattern,
    pattern_node::PatternNode,
    tokenizer::{tokenize, tokenize_value, BracketToken},
};

pub fn parse(pattern_str: &str) -> Result<Pattern, ParsingError> {
    let pattern_str = pattern_str.trim();
    let mut tokens = tokenize(pattern_str)?;

    // The tokenizer stops at the first top level colon, so the value, when there is one, is the
    // last token, with the colon right before it.
    let (path_end, value_token, has_colon) = match tokens.last() {
        Some(Token::Value(tc)) => (tokens.len() - 2, Some(tc), true),
        Some(Token::Colon) => (tokens.len() - 1, None, true),
        _ => (tokens.len(), None, false),
    };
    let mut value = value_token.map(add_value_wildcards);
    add_path_wildcards(&mut tokens[..path_end]);

    let (path, start_at_root) = parse_path(&tokens[..path_end])?;

    // Parse pattern as a value if it doesn't have a `:`, and is not explicitly a path, that is,
    // doesn't start with a `.`, `$`, or `[`
    if !has_colon
        && !start_at_root
        && !matches!(tokens.first(), Some(Token::BracketExpr(_) | &Token::Dot))
    {
        value = parse_bare_value(pattern_str)?;
    }
    let or = !has_colon && value.is_some();

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
            Token::Colon | Token::Value(_) => unreachable!(),
        };

        if let Some(t) = token {
            pattern_vec.push(t)
        }
    }

    Ok((pattern_vec, start_at_root))
}

/// Puts a wildcard at each end of the path, so that it matches partially. Quoted ends are matched
/// exactly, so they are left alone.
fn add_path_wildcards(tokens: &mut [Token]) {
    if let Some(Token::Text(tc)) = tokens.first_mut() {
        if !tc.quoted {
            tc.str = format!("*{}", tc.str);
        }
    }

    if let Some(Token::Text(tc)) = tokens.last_mut() {
        if !tc.quoted {
            tc.str = format!("{}*", tc.str);
        }
    }
}

/// Puts a wildcard at each end of the value, so that it matches partially. Quoted values are
/// matched exactly, so they are left alone.
///
/// The value gets its own pair rather than sharing the path's, so that `.a: b` looks for `*b*`
/// and not for `b*`.
fn add_value_wildcards(value: &TextContent) -> String {
    if value.quoted {
        value.str.clone()
    } else {
        format!("*{}*", value.str)
    }
}

fn parse_bare_value(pattern_str: &str) -> Result<Option<String>, ParsingError> {
    let chars: Vec<char> = pattern_str.chars().collect();

    Ok(tokenize_value(&chars, 0)?.as_ref().map(add_value_wildcards))
}

fn parse_brackets(tokens: &[BracketToken]) -> Result<PatternNode, ParsingError> {
    let colon_indices = token_indices(tokens, BracketToken::Colon);
    let comma_indices = token_indices(tokens, BracketToken::Comma);

    // [] or []
    if tokens.is_empty()
        || matches!(
            tokens,
            [BracketToken::Text(TextContent { str, quoted })] if str == "*" && !*quoted
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
