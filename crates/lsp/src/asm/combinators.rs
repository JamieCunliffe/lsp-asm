use std::num::ParseIntError;

use crate::asm::config::FileType;

use super::config::ParserConfig;

use super::ast::SyntaxKind;

use either::Either;

use nom::bytes::complete::{is_not, tag, take_while};
use nom::character::is_hex_digit;
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::sequence::delimited;
use rowan::{GreenNode, GreenToken, NodeOrToken};

type Span<'a> = super::span::Span<'a, &'a InternalSpanConfig<'a>>;
type NomResultElement<'a> = nom::IResult<Span<'a>, NodeOrToken<GreenNode, GreenToken>>;
type SyntaxLookupMap =
    std::collections::HashMap<char, Either<SyntaxKind, Box<dyn Fn(Span) -> NomResultElement>>>;

struct InternalSpanConfig<'a> {
    config: &'a ParserConfig,
    lookup: &'a SyntaxLookupMap,
}

impl<'a> PartialEq for InternalSpanConfig<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}
impl<'a> std::fmt::Debug for InternalSpanConfig<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.config)
    }
}
impl<'a> InternalSpanConfig<'a> {
    fn new(config: &'a ParserConfig, lookup: &'a SyntaxLookupMap) -> Self {
        Self { config, lookup }
    }
}

impl<'a> Span<'a> {
    pub(crate) fn config(&self) -> &ParserConfig {
        &self.extra().config
    }
}

/// Performs the checks to exit out of a end many function
macro_rules! end_many0(
    ($e:expr) => (
        if $e.is_empty() {
            return nom::lib::std::result::Result::Err(nom::Err::Error(($e, ErrorKind::Many0)));
        }
    ));

macro_rules! process_comment(
    ($e:expr) => (
        if $e.as_str().starts_with(&$e.config().comment_start) {
            return parse_comment($e);
        }
    ));

/// Entry point into the parser
/// # Arguments
/// * `data` The data that should be parsed
/// * `config` the parser configuration
///
/// # Returns
/// A vector containing all the nodes or tokens for the input, this will *NOT* include a root node.
pub(crate) fn parse<'a>(
    data: &'a str,
    config: &'a ParserConfig,
) -> Vec<NodeOrToken<GreenNode, GreenToken>> {
    let lookup = create_syntax_lookup(&config);
    let internal = InternalSpanConfig::new(config, &lookup);
    let data = Span::new(data, &internal);

    let (data, mut additional) = match config.file_type {
        crate::asm::config::FileType::Assembly => (data, Vec::new()),
        crate::asm::config::FileType::ObjDump => parse_objdump_header(data).unwrap(),
    };

    let result = many0(parse_next)(data);
    let (remaining, mut nodes) = match result {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse due to error: {:#?}", e),
    };

    debug!("Parsed assembly with data remaining: {:#?}", remaining);

    additional.append(&mut nodes);
    additional
}

fn parse_objdump_header(expr: Span) -> nom::IResult<Span, Vec<NodeOrToken<GreenNode, GreenToken>>> {
    let (remaining, whitespace) = skip_whitespace(expr, true)?;
    let (remaining, format) = take_while(|a| a != '\n')(remaining)?;

    let format = GreenToken::new(SyntaxKind::METADATA.into(), format.as_str());
    Ok((remaining, vec![whitespace, format.into()]))
}

fn parse_objdump_line_start(
    expr: Span,
) -> nom::IResult<Span, Vec<NodeOrToken<GreenNode, GreenToken>>> {
    let (remaining, whitespace) = skip_whitespace(expr, true)?;
    let (remaining, offset) = take_while(is_hex)(remaining)?;

    let mut tokens = Vec::new();
    if whitespace.text_len() > 0.into() {
        tokens.push(whitespace);
    }

    tokens.push(GreenToken::new(SyntaxKind::METADATA.into(), offset.as_str()).into());

    // If the next char is a : then we will be parsing the instruction encoding
    let remaining = if remaining.as_str().starts_with(':') {
        let (remaining, colon) = take_while(|a| a == ':')(remaining)?;
        tokens.push(GreenToken::new(SyntaxKind::METADATA.into(), colon.as_str()).into());

        let (remaining, whitespace) = skip_whitespace(remaining, false)?;
        tokens.push(whitespace);

        let (remaining, encoding) = take_while(|a| is_hex(a) || a == ' ')(remaining)?;
        tokens.push(GreenToken::new(SyntaxKind::METADATA.into(), encoding.as_str()).into());

        let (remaining, whitespace) = skip_whitespace(remaining, false)?;
        tokens.push(whitespace);

        remaining
    } else {
        let (remaining, ws) = skip_whitespace(remaining, false)?;
        tokens.push(ws);
        remaining
    };

    Ok((remaining, tokens))
}

fn objdump_angle_brackets(expr: Span) -> NomResultElement {
    if expr.as_str().ends_with(':') {
        let (remaining, token) = take_while(|a| a != '\n')(expr)?;
        Ok((remaining, span_to_token(&token).into()))
    } else {
        let (remaining, brackets) =
            parse_brackets(expr, (SyntaxKind::L_ANGLE, SyntaxKind::R_ANGLE))?;

        let meta = GreenNode::new(SyntaxKind::METADATA.into(), vec![brackets]);
        Ok((remaining, meta.into()))
    }
}

/// To be called by a many0 function to perform the processing.
/// Each call to this should produce a statement.
fn parse_next(expr: Span) -> NomResultElement {
    end_many0!(expr);

    let first = expr.chars().next().unwrap_or('\0');

    match first {
        ' ' => skip_whitespace(expr, true),
        '\n' => skip_whitespace(expr, true),
        '\t' => skip_whitespace(expr, true),
        // '\0' => nom::lib::std::result::Result::Err(nom::Err::Error((expr, ErrorKind::Many0))),
        'D' if expr.config().file_type == FileType::ObjDump
            && expr.as_str().starts_with("Disassembly of section ") =>
        {
            let (remaining, token) = take_while(|a| a != '\n')(expr)?;
            let token = GreenToken::new(SyntaxKind::METADATA.into(), token.as_str());
            Ok((remaining, token.into()))
        }
        _ => {
            // Extract the current line from the input for processing
            process_comment!(expr);
            let (remaining, expr) = take_while(|a| a != '\n')(expr)?;

            let (expr, mut additional) = match expr.config().file_type {
                FileType::Assembly => (expr, Vec::new()),
                FileType::ObjDump => parse_objdump_line_start(expr)?,
            };

            let (_, mut tokens) = match many0(parse_line)(expr) {
                Ok(a) => a,
                Err(e) => {
                    error!("Failed to parse line: {:#?}", e);
                    return Err(e);
                }
            };

            let mut kind = SyntaxKind::INSTRUCTION;
            if let Some(first) = tokens.first() {
                if let Some(token) = first.as_token() {
                    if token.kind() == SyntaxKind::LABEL.into() {
                        kind = if token.text().starts_with('.') {
                            SyntaxKind::LOCAL_LABEL
                        } else {
                            SyntaxKind::LABEL
                        };
                    } else {
                        if token.text().starts_with('.') {
                            kind = SyntaxKind::DIRECTIVE;
                        }

                        let new_token =
                            GreenToken::new(SyntaxKind::MNEMONIC.into(), token.text()).into();
                        let _ = std::mem::replace(&mut tokens[0], new_token);
                    }
                }
            }
            additional.append(&mut tokens);
            let token = GreenNode::new(kind.into(), additional);
            Ok((remaining, token.into()))
        }
    }
}

/// Parse a single instruction, this is probably a line but might not be.
/// This function is designed to used with a many0 expression
fn parse_line(expr: Span) -> NomResultElement {
    end_many0!(expr);

    let tokens = expr.extra().lookup;
    let first = expr.chars().next().unwrap_or('\0');
    match tokens.get(&first) {
        Some(Either::Left(kind)) => {
            let (remaining, val) = take_while(|a| a == first)(expr)?;
            let token = GreenToken::new((*kind).into(), val.as_str());
            Ok((remaining, token.into()))
        }
        Some(Either::Right(action)) => action(expr),
        None => {
            // If we start with a comment parse it.
            process_comment!(expr);

            let (remaining, token) = take_while(|a: char| tokens.get(&a).is_none())(expr)?;
            let token = span_to_token(&token);
            Ok((remaining, token.into()))
        }
    }
}

/// Converts the span into a GreenToken
fn span_to_token(token: &Span) -> GreenToken {
    if super::registers::is_register(token.as_str(), &token.extra().config) {
        GreenToken::new(SyntaxKind::REGISTER.into(), token.as_str())
    } else if is_numeric(token.as_str()) {
        GreenToken::new(SyntaxKind::NUMBER.into(), token.as_str())
    } else if token.as_str().ends_with(':') {
        GreenToken::new(SyntaxKind::LABEL.into(), token.as_str())
    } else {
        GreenToken::new(SyntaxKind::TOKEN.into(), token.as_str())
    }
}

/// Skip over any characters that are considered to be whitespace
fn skip_whitespace(remaining: Span, skip_newlines: bool) -> NomResultElement {
    let (remaining, ws) =
        take_while(|a| a == ' ' || a == '\t' || (a == '\n' && skip_newlines))(remaining)?;

    let token = GreenToken::new(SyntaxKind::WHITESPACE.into(), ws.as_str());

    Ok((remaining, token.into()))
}

/// Tests is a string is to be considered numeric, this will account for various
/// numeric prefixes that are legal `parse_number`
fn is_numeric(token: &str) -> bool {
    parse_number(token).is_ok()
}

/// Parse a number into an i128, this will account for any prefixes and use them.
/// a $ or ' will be ignored and skipped any number prefixed with 0x will be
/// parsed as a base 16 number
pub(crate) fn parse_number(token: &str) -> Result<i128, ParseIntError> {
    let token = token.trim_start_matches(|c: char| ['$', '#'].contains(&c));
    if let Some(token) = token.strip_prefix("0x") {
        i128::from_str_radix(token, 16)
    } else {
        i128::from_str_radix(token, 10)
    }
}

/// Parse any expression that is contained within a set of brackets
/// This will recursively apply the normal parsing rules to create arrays of expressions that are nested
/// # Arguments
/// * `remaining` The data to parse
/// * `pair` a tuple containing the brackets are are looking for e.g. `('[', ']')`
fn parse_brackets(remaining: Span, tokens: (SyntaxKind, SyntaxKind)) -> NomResultElement {
    let pair = match tokens {
        (SyntaxKind::L_PAREN, SyntaxKind::R_PAREN) => ("(", ")"),
        (SyntaxKind::L_SQ, SyntaxKind::R_SQ) => ("[", "]"),
        (SyntaxKind::L_CURLY, SyntaxKind::R_CURLY) => ("{", "}"),
        (SyntaxKind::L_ANGLE, SyntaxKind::R_ANGLE) => ("<", ">"),
        _ => panic!("Unexpected bracket type"),
    };

    let (remaining, inner) = get_bracket_span(remaining, pair)?;
    let (_, inner) = many0(parse_line)(inner)?;

    let open = GreenToken::new(tokens.0.into(), pair.0).into();
    let close = GreenToken::new(tokens.1.into(), pair.1).into();
    let inner = vec![vec![open], inner, vec![close]]
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let root = GreenNode::new(SyntaxKind::BRACKETS.into(), inner);
    Ok((remaining, root.into()))
}

/// Skip the parser to the end of the line and generate a `TokenValue::Comment` with the contents
/// of `remaining`
/// # Arguments
/// * `remaining` the string containing the comment. It's the callers responsibility for ensuring
/// that this string is actually a comment string.
fn parse_comment(remaining: Span) -> NomResultElement {
    assert!(remaining
        .as_str()
        .starts_with(&remaining.config().comment_start));

    let (remaining, comment) = take_while(|a| a != '\n')(remaining)?;
    let root = GreenToken::new(SyntaxKind::COMMENT.into(), comment.as_str());

    Ok((remaining, root.into()))
}

/// Gets the data between a pair of brackets, this will account for nested
/// brackets
fn get_bracket_span<'a>(
    remaining: Span<'a>,
    pair: (&'a str, &'a str),
) -> nom::IResult<Span<'a>, Span<'a>> {
    let (open, close) = pair;
    delimited(
        tag(open),
        take_until_balanced(open.chars().next().unwrap(), close.chars().next().unwrap()),
        tag(close),
    )(remaining)
}

/// Utility function to take data until the brackets have been balanced.
fn take_until_balanced<T, Input, Error: nom::error::ParseError<Input>>(
    open: T,
    close: T,
) -> impl Fn(Input) -> nom::IResult<Input, Input, Error>
where
    Input: nom::InputIter + nom::InputTake + nom::InputIter<Item = T>,
    T: Clone + Copy + PartialEq,
{
    move |i: Input| {
        let mut ctr = 1;
        for (index, val) in i.iter_indices() {
            if val == open {
                ctr += 1;
            } else if val == close {
                ctr -= 1;
            }

            if ctr == 0 {
                return Ok(i.take_split(index));
            }
        }

        Err(nom::Err::Error(Error::from_error_kind(
            i,
            ErrorKind::TakeUntil,
        )))
    }
}

/// Parse a string constant
fn parse_string(remaining: Span) -> nom::IResult<Span, String> {
    let (remaining, inner) = match delimited(tag("\""), is_not("\""), tag("\""))(remaining) {
        Ok(a) => a,
        nom::lib::std::result::Result::Err(nom::Err::Error((r, kind)))
            if kind == ErrorKind::IsNot =>
        {
            // An empty string will give an error due to the sep part of delimited being the char we are looking for, we can
            // handle that case by just returning an empty string, but we have to ensure we update the parser to skip over
            // the "" that would be contained within.

            let (remaining, _quote) = take_while(|a| a == '"')(r)?;

            return Ok((remaining, String::from(r#""""#)));
        }
        Err(e) => return Err(e),
    };

    Ok((remaining, format!(r#""{}""#, inner.as_str())))
}

/// Parse a minus number, this will start at the - token and use the standard
/// rules defined from `is_numeric`, if the token isn't number then an operator
/// of '-' is returned
fn parse_minus(expr: Span) -> NomResultElement {
    let tokens = expr.extra().lookup;

    let (remaining_operator, minus_token) = take_while(|a| a == '-')(expr)?;
    let (remaining_neg, token) =
        take_while(|a: char| tokens.get(&a).is_none())(remaining_operator.clone())?;

    if is_numeric(token.as_str()) {
        let token = GreenToken::new(SyntaxKind::NUMBER.into(), &format!("-{}", token.as_str()));
        Ok((remaining_neg, token.into()))
    } else {
        let token = GreenToken::new(SyntaxKind::OPERATOR.into(), minus_token.as_str());
        Ok((remaining_operator, token.into()))
    }
}

macro_rules! hashmap {
    ($t:ty; $($key:expr => $value:expr,)+) => { hashmap!( $t; $($key => $value),+) };
    ($t:ty; $($key:expr => $value:expr),*) => {
        {
            let mut _map: $t = ::std::collections::HashMap::new();
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

fn create_syntax_lookup(config: &ParserConfig) -> SyntaxLookupMap {
    let mut map = hashmap! { SyntaxLookupMap;
               ',' => Either::Left(SyntaxKind::COMMA),
               '-' => Either::Right(Box::new(parse_minus)),
               ' ' => Either::Right(Box::new(|expr| skip_whitespace(expr, false))),
               '\t' => Either::Right(Box::new(|expr| skip_whitespace(expr, false))),
               '\n' => Either::Right(Box::new(|expr| skip_whitespace(expr, false))),
               '(' => Either::Right(Box::new(|expr| {
                   parse_brackets(expr, (SyntaxKind::L_PAREN, SyntaxKind::R_PAREN))
               })),
               '[' => Either::Right(Box::new(|expr| {
                   parse_brackets(expr, (SyntaxKind::L_SQ, SyntaxKind::R_SQ))
               })),
               '{' => Either::Right(Box::new(|expr| {
                   parse_brackets(expr, (SyntaxKind::L_CURLY, SyntaxKind::R_CURLY))
               })),
               '"' => Either::Right(Box::new(|expr| {
                   let (remaining, str) = parse_string(expr)?;
                   Ok((
                       remaining,
                       GreenToken::new(SyntaxKind::STRING.into(), &str).into(),
                   ))
               })),
    };
    if config.file_type == FileType::ObjDump {
        map.insert('<', Either::Right(Box::new(objdump_angle_brackets)));
    }
    map
}

fn is_hex(data: char) -> bool {
    is_hex_digit(data as u8)
}
