use crate::ParsedData;

use super::builder::Builder;
use super::config::ParserConfig;

use base::{Architecture, FileType};
use either::Either;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::is_hex_digit;
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, InputLength, InputTake};
use rowan::GreenNode;
use std::num::{ParseFloatError, ParseIntError};
use syntax::ast::SyntaxKind;

type Span<'a> = super::span::Span<'a, &'a InternalSpanConfig<'a>>;
type NomResultElement<'a> = nom::IResult<Span<'a>, ()>;

struct InternalSpanConfig<'a> {
    config: &'a ParserConfig,
    builder: &'a Builder,
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
    fn new(config: &'a ParserConfig, builder: &'a Builder) -> Self {
        Self { config, builder }
    }
}

impl<'a> Span<'a> {
    pub(crate) fn config(&self) -> &ParserConfig {
        self.extra().config
    }
    pub(self) fn start_node(&self, kind: SyntaxKind) {
        self.extra().builder.start_node(kind)
    }
    pub(self) fn token(&self, kind: SyntaxKind, text: &str) {
        self.extra().builder.token(kind, text)
    }
    pub(self) fn finish_node(&self) {
        self.extra().builder.finish_node();
    }
    pub(self) fn finish(&self) -> GreenNode {
        self.extra().builder.finish()
    }
    pub(self) fn current_indent_is_kind(&self, kind: SyntaxKind) -> bool {
        self.extra().builder.current_indent_is_kind(kind)
    }
    pub(self) fn last_kind(&self) -> SyntaxKind {
        self.extra().builder.last_kind()
    }
}

/// Performs the checks to exit out of a end many function
macro_rules! end_many0(
    ($e:expr) => (
        if $e.is_empty() {
            return nom::lib::std::result::Result::Err(nom::Err::Error(nom::error::Error::new($e, ErrorKind::Many0)));
        }
    ));

macro_rules! process_comment(
    ($e:expr) => (
        if $e.as_str().starts_with(&$e.config().comment_start) {
            return parse_comment($e);
        }

        if matches!($e.extra().config.file_type, FileType::ObjDump) && $e.as_str().starts_with(';') {
            return parse_comment($e);
        }

        if $e.as_str().starts_with("/*") {
            return parse_multiline_comment($e);
        }
    ));

pub(crate) fn parse<'a>(data: &'a str, config: &'a ParserConfig) -> ParsedData {
    let builder = Builder::new(data.len() / 4);
    let internal = InternalSpanConfig::new(config, &builder);
    let data = Span::new(data, &internal);

    data.start_node(SyntaxKind::ROOT);
    let data = match config.file_type {
        FileType::Assembly => data,
        FileType::ObjDump => parse_objdump_header(data).unwrap().0,
    };

    let result = many0(parse_next)(data);
    let (remaining, _) = match result {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse due to error: {:#?}", e),
    };

    debug!("Parsed assembly with data remaining: {:#?}", remaining);

    ParsedData {
        root: remaining.finish(),
        alias: remaining.extra().builder.alias.take(),
    }
}

fn parse_objdump_header(expr: Span) -> nom::IResult<Span, ()> {
    let (remaining, _) = skip_whitespace(expr, true)?;
    let (remaining, format) = take_while(|a| a != '\n')(remaining)?;

    remaining.token(SyntaxKind::METADATA, format.as_str());

    Ok((remaining, ()))
}

fn parse_objdump_line_start(expr: Span) -> nom::IResult<Span, ()> {
    let (remaining, _) = skip_whitespace(expr, true)?;
    let (remaining, offset) = take_while(is_hex)(remaining)?;

    remaining.token(SyntaxKind::METADATA, offset.as_str());

    // If the next char is a : then we will be parsing the instruction encoding
    let remaining = if remaining.as_str().starts_with(':') {
        let (remaining, colon) = take_while(|a| a == ':')(remaining)?;
        remaining.token(SyntaxKind::METADATA, colon.as_str());

        let (remaining, _) = skip_whitespace(remaining, false)?;

        let (remaining, encoding) = take_while(|a| is_hex(a) || a == ' ')(remaining)?;
        remaining.token(SyntaxKind::METADATA, encoding.as_str());

        let (remaining, _) = skip_whitespace(remaining, false)?;

        remaining
    } else {
        let (remaining, _) = skip_whitespace(remaining, false)?;
        remaining
    };

    Ok((remaining, ()))
}

fn objdump_angle_brackets(expr: Span) -> NomResultElement {
    if expr.as_str().ends_with(':') {
        let (remaining, token) = take_while(|a| a != '\n')(expr)?;
        span_to_token(&token);
        Ok((remaining, ()))
    } else {
        expr.start_node(SyntaxKind::METADATA);
        let (remaining, _) = parse_brackets(expr, (SyntaxKind::L_ANGLE, SyntaxKind::R_ANGLE))?;
        remaining.finish_node();

        Ok((remaining, ()))
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
            remaining.token(SyntaxKind::METADATA, token.as_str());
            Ok((remaining, ()))
        }
        _ => {
            process_comment!(expr);

            // Extract the current line from the input for processing
            let (remaining, expr) = take_while(|a| a != '\n')(expr)?;

            process_line(expr)?;

            Ok((remaining, ()))
        }
    }
}

fn process_line(expr: Span) -> NomResultElement {
    process_comment!(expr);
    // Check to see if we need to end any nodes before processing this one
    let kind = {
        let kind = pre_process_next(expr.as_str(), expr.extra().config);

        if matches!(kind, SyntaxKind::LOCAL_LABEL | SyntaxKind::LABEL)
            && expr.current_indent_is_kind(SyntaxKind::LOCAL_LABEL)
        {
            expr.finish_node();
        }

        if matches!(kind, SyntaxKind::LABEL) && expr.current_indent_is_kind(SyntaxKind::LABEL) {
            expr.finish_node();
        }

        kind
    };
    expr.start_node(kind);
    let expr = match expr.config().file_type {
        FileType::Assembly => expr,
        FileType::ObjDump => parse_objdump_line_start(expr)?.0,
    };
    let (expr, _) = skip_whitespace(expr, false)?;
    let (expr, token) = take_while(|a: char| !a.is_whitespace())(expr)?;
    let actual_kind = start_kind(token.as_str());
    assert_eq!(kind, actual_kind);

    let expr = if matches!(kind, SyntaxKind::DIRECTIVE | SyntaxKind::INSTRUCTION) {
        expr.token(SyntaxKind::MNEMONIC, token.as_str());
        expr
    } else {
        expr.token(SyntaxKind::LABEL, token.as_str());
        let (expr, _) = skip_whitespace(expr, false)?;
        if !expr.as_str().is_empty() {
            return process_line(expr);
        }
        expr
    };

    let (x, _) = match many0(parse_line)(expr) {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to parse line: {:#?}", e);
            return Err(e);
        }
    };
    if x.current_indent_is_kind(SyntaxKind::EXPR) {
        x.finish_node();
    }
    if matches!(kind, SyntaxKind::DIRECTIVE | SyntaxKind::INSTRUCTION) {
        x.finish_node();
    }

    assert!(x.as_str().is_empty());

    Ok((x, ()))
}

fn pre_process_next(line: &str, config: &ParserConfig) -> SyntaxKind {
    let token = match config.file_type {
        FileType::Assembly => line.trim_start_matches(|a| a == ' ' || a == '\t'),
        FileType::ObjDump => {
            let remaining = line.trim_start_matches(|a| a == ' ');
            let remaining = remaining.trim_start_matches(|a| is_hex(a) || a == ' ');

            if remaining.starts_with(':') {
                remaining
                    .trim_start_matches(|a| a == ':')
                    .trim_start_matches(|a| a == '\t')
                    .trim_start_matches(|a| is_hex(a) || a == ' ')
                    .trim_start()
            } else {
                remaining.trim_start_matches(|a| a == ' ')
            }
        }
    };

    let mut split = token.split(|a: char| a.is_whitespace());
    start_kind(split.next().unwrap_or(token))
}

fn start_kind(token: &str) -> SyntaxKind {
    if token.ends_with(':') {
        if token.starts_with('.') {
            SyntaxKind::LOCAL_LABEL
        } else {
            SyntaxKind::LABEL
        }
    } else if token.starts_with('.') {
        SyntaxKind::DIRECTIVE
    } else {
        SyntaxKind::INSTRUCTION
    }
}

/// Parse a single instruction, this is probably a line but might not be.
/// This function is designed to used with a many0 expression
fn parse_line(expr: Span) -> NomResultElement {
    end_many0!(expr);
    let config = expr.extra().config;
    let first = expr.chars().next().unwrap_or('\0');
    match get_action(first, config) {
        Some(Either::Left(kind)) => {
            let (remaining, val) = take_while(|a| a == first)(expr)?;
            remaining.token(kind, val.as_str());
            Ok((remaining, ()))
        }
        Some(Either::Right(action)) => action(expr),
        None => {
            // If we start with a comment parse it.
            process_comment!(expr);

            let (remaining, token) = take_while(|a: char| !is_special_char(a, config))(expr)?;
            span_to_token(&token);
            Ok((remaining, ()))
        }
    }
}

pub fn register_name(name: &str) -> String {
    name.strip_prefix('%').unwrap_or(name).to_lowercase()
}

/// Determine if `name` is a valid register
fn is_register(name: &str, config: &ParserConfig) -> bool {
    if let Some(registers) = config.registers {
        let name = register_name(name);
        registers
            .iter()
            .any(|register| register.names.contains(&name.as_str()))
    } else {
        false
    }
}

/// Converts the span into a GreenToken
fn span_to_token(token: &Span) {
    if is_register(token.as_str(), token.extra().config) {
        token.token(SyntaxKind::REGISTER, token.as_str());
    } else if is_numeric(token.as_str()) {
        token.token(SyntaxKind::NUMBER, token.as_str());
    } else if is_floating_point(token.as_str()) {
        token.token(SyntaxKind::FLOAT, token.as_str());
    } else if token.as_str().ends_with(':') {
        token.token(SyntaxKind::LABEL, token.as_str());
    } else if token.config().architecture == Architecture::AArch64 && token.as_str() == ".req" {
        token.extra().builder.change_node_kind(SyntaxKind::ALIAS);
        token
            .extra()
            .builder
            .change_previous_token_kind(1, SyntaxKind::REGISTER_ALIAS);
        token.token(SyntaxKind::MNEMONIC, token.as_str());
    } else if token.as_str().eq_ignore_ascii_case(".equ")
        || token.as_str().eq_ignore_ascii_case("equ")
    {
        token
            .extra()
            .builder
            .change_node_kind(SyntaxKind::CONST_DEF);
        token
            .extra()
            .builder
            .change_previous_token_kind(1, SyntaxKind::NAME);
        token.token(SyntaxKind::MNEMONIC, token.as_str());
        token.start_node(SyntaxKind::EXPR);
    } else if let Some(kind) = token.extra().builder.alias().get_kind(token.as_str()) {
        token.token(kind, token.as_str());
    } else {
        token.token(SyntaxKind::TOKEN, token.as_str());
    }
}

/// Skip over any characters that are considered to be whitespace
fn skip_whitespace(remaining: Span, skip_newlines: bool) -> NomResultElement {
    let (remaining, ws) =
        take_while(|a| a == ' ' || a == '\t' || (a == '\n' && skip_newlines))(remaining)?;

    if !ws.as_str().is_empty() {
        remaining.token(SyntaxKind::WHITESPACE, ws.as_str());
    }

    Ok((remaining, ()))
}

/// Tests if a string is to be considered numeric, this will account for various
/// numeric prefixes that are legal in `parse_number`
fn is_numeric(token: &str) -> bool {
    parse_number(token).is_ok()
}

/// Tests if a string is a floating point number
fn is_floating_point(token: &str) -> bool {
    parse_float(token).is_ok()
}

/// Parse a number into an i128, this will account for any prefixes and use them.
/// a $ or # will be ignored and skipped any number prefixed with 0x will be
/// parsed as a base 16 number
pub fn parse_number(token: &str) -> Result<i128, ParseIntError> {
    let token = token.trim_start_matches(|c: char| ['$', '#'].contains(&c));
    if let Some(token) = token.strip_prefix("0x") {
        i128::from_str_radix(token, 16)
    } else {
        token.parse::<i128>()
    }
}

pub(crate) fn parse_float(token: &str) -> Result<f64, ParseFloatError> {
    let token = token.trim_start_matches(|c: char| ['$', '#'].contains(&c));
    token.parse::<f64>()
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

    let span = get_bracket_span(remaining.clone(), pair);

    if let Ok((remaining, inner)) = span {
        remaining.start_node(SyntaxKind::BRACKETS);
        remaining.token(tokens.0, pair.0);
        many0(parse_line)(inner)?;
        remaining.token(tokens.1, pair.1);
        remaining.finish_node();

        Ok((remaining, ()))
    } else {
        remaining.start_node(SyntaxKind::BRACKETS);
        remaining.token(tokens.0, pair.0);
        let (remaining, _) = remaining.take_split(1);
        let (remaining, _) = many0(parse_line)(remaining)?;
        remaining.finish_node();
        Ok((remaining, ()))
    }
}

/// Skip the parser to the end of the line and generate a `TokenValue::Comment` with the contents
/// of `remaining`
/// # Arguments
/// * `remaining` the string containing the comment. It's the callers responsibility for ensuring
/// that this string is actually a comment string.
fn parse_comment(remaining: Span) -> NomResultElement {
    let (remaining, comment) = take_while(|a| a != '\n')(remaining)?;
    remaining.token(SyntaxKind::COMMENT, comment.as_str());

    Ok((remaining, ()))
}

fn parse_multiline_comment(remaining: Span) -> NomResultElement {
    assert!(remaining.as_str().starts_with("/*"));

    let (remaining, comment) = match take_until("*/")(remaining) {
        Ok((remaining, comment)) => {
            let (remaining, end) = remaining.take_split(2);
            let comment = format!("{}{}", comment.as_str(), end.as_str());
            (remaining, comment)
        }
        nom::lib::std::result::Result::Err(nom::Err::Error((r, ErrorKind::TakeUntil))) => {
            let (remaining, comment) = take_while(|a| a != '\0')(r)?;
            (remaining, comment.as_str().to_string())
        }
        e => panic!("Unexpected result for multiline comment: {:#?}", e),
    };

    remaining.token(SyntaxKind::COMMENT, &comment);

    Ok((remaining, ()))
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

pub fn take_while_skip_first<F, T, Input, Error: nom::error::ParseError<Input>>(
    cond: F,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
    Input: nom::InputTakeAtPosition<Item = T>
        + nom::InputIter<Item = T>
        + nom::InputTake
        + InputLength,
    F: Fn(T) -> bool,
{
    move |i: Input| {
        for (index, val) in i.iter_indices() {
            if index > 0 && !cond(val) {
                return Ok(i.take_split(index));
            }
        }

        Ok(i.take_split(i.input_len()))
    }
}

fn str_parse<T, Input>(data: Input) -> IResult<Input, Input>
where
    Input: nom::InputTake + nom::InputIter<Item = T>,
    T: Into<char>,
{
    let mut escaped = false;
    for (i, ch) in data.iter_indices() {
        match ch.into() {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return Ok(data.take_split(i)),
            _ => escaped = false,
        }
    }
    Err(nom::Err::Incomplete(nom::Needed::Unknown))
}

/// Parse a string constant
fn parse_string(remaining: Span) -> nom::IResult<Span, String> {
    let (remaining, inner) = terminated(preceded(tag("\""), str_parse), tag("\""))(remaining)?;

    Ok((remaining, format!(r#""{}""#, inner.as_str())))
}

/// Parse a minus number, this will start at the - token and use the standard
/// rules defined from `is_numeric`, if the token isn't number then an operator
/// of '-' is returned
fn parse_minus(expr: Span) -> NomResultElement {
    let config = expr.extra().config;
    let (remaining_operator, minus_token) = take_while(|a| a == '-')(expr)?;
    let (remaining_neg, token) =
        take_while(|a: char| !is_special_char(a, config))(remaining_operator.clone())?;
    if is_numeric(token.as_str()) {
        remaining_neg.token(SyntaxKind::NUMBER, &format!("-{}", token.as_str()));
        Ok((remaining_neg, ()))
    } else {
        remaining_operator.token(SyntaxKind::OPERATOR, minus_token.as_str());
        Ok((remaining_operator, ()))
    }
}

type ProcessFunction = Box<dyn Fn(Span) -> NomResultElement>;
#[inline]
fn get_action(c: char, config: &ParserConfig) -> Option<Either<SyntaxKind, ProcessFunction>> {
    match c {
        ',' => Some(Either::Left(SyntaxKind::COMMA)),
        '+' => Some(Either::Left(SyntaxKind::OPERATOR)),
        '-' => Some(Either::Right(Box::new(parse_minus))),
        ' ' => Some(Either::Right(Box::new(|expr| skip_whitespace(expr, false)))),
        '\t' => Some(Either::Right(Box::new(|expr| skip_whitespace(expr, false)))),
        '\n' => Some(Either::Right(Box::new(|expr| skip_whitespace(expr, false)))),
        '(' => Some(Either::Right(Box::new(|expr| {
            parse_brackets(expr, (SyntaxKind::L_PAREN, SyntaxKind::R_PAREN))
        }))),
        '[' => Some(Either::Right(Box::new(|expr| {
            parse_brackets(expr, (SyntaxKind::L_SQ, SyntaxKind::R_SQ))
        }))),
        '{' => Some(Either::Right(Box::new(|expr| {
            parse_brackets(expr, (SyntaxKind::L_CURLY, SyntaxKind::R_CURLY))
        }))),
        '"' => Some(Either::Right(Box::new(|expr| {
            let (remaining, str) = parse_string(expr)?;
            remaining.token(SyntaxKind::STRING, &str);
            Ok((remaining, ()))
        }))),
        '@' => Some(Either::Right(Box::new(|expr| {
            let config = expr.extra().config;
            let (remaining, token) =
                take_while_skip_first(|a: char| !is_special_char(a, config))(expr)?;
            let kind = (remaining.last_kind() == SyntaxKind::TOKEN)
                .then(|| SyntaxKind::RELOCATION)
                .unwrap_or(SyntaxKind::TOKEN);

            remaining.token(kind, token.as_str());
            Ok((remaining, ()))
        }))),
        '<' if config.file_type == FileType::ObjDump => {
            Some(Either::Right(Box::new(objdump_angle_brackets)))
        }
        '#' if config.architecture == Architecture::AArch64 => {
            Some(Either::Right(Box::new(|expr| {
                let (remaining, str) = take_while(|a| a != ' ' && a != ',')(expr)?;
                let token = span_to_token(&str);
                Ok((remaining, token))
            })))
        }
        ':' if config.architecture == Architecture::AArch64 => {
            Some(Either::Right(Box::new(|expr| {
                let config = expr.extra().config;
                let split = expr.as_str()[1..]
                    .find(|c| is_special_char(c, config))
                    .map(|i| i + 1)
                    .unwrap_or_else(|| expr.as_str().len());

                let (kind, split) = if expr.as_str().get(split..=split) == Some(":") {
                    (SyntaxKind::RELOCATION, split + 1)
                } else {
                    (SyntaxKind::TOKEN, split)
                };

                let (remaining, relocation) = expr.take_split(split);
                remaining.token(kind, relocation.as_str());

                Ok((remaining, ()))
            })))
        }
        _ => None,
    }
}

#[inline]
fn is_special_char(c: char, config: &ParserConfig) -> bool {
    match c {
        ',' | '+' | '-' | ' ' | '\t' | '\n' | '(' | '[' | '{' | '"' | '@' => true,
        '<' if config.file_type == FileType::ObjDump => true,
        '#' | ':' if config.architecture == Architecture::AArch64 => true,
        _ => false,
    }
}

fn is_hex(data: char) -> bool {
    is_hex_digit(data as u8)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_is_numeric() {
        assert_eq!(true, is_numeric("#-42"));
        assert_eq!(true, is_numeric("0x123456789ABCDEF"));
        assert_eq!(true, is_numeric("0x123456789abcdef"));
    }

    #[test]
    fn test_is_numeric_float() {
        assert_eq!(true, is_floating_point("#1.00000"));
    }

    #[test]
    fn objdump_peek() {
        let data = r#"00000000002015a0 <_start>:
  2015a0:	f3 0f 1e fa          	endbr64
  2015a4:	31 ed                	xor    %ebp,%ebp
  210cac:	00000000 	.inst	0x00000000 ; undefined"#;
        let mut lines = data.split('\n');
        let config = ParserConfig {
            architecture: Architecture::X86_64,
            file_type: FileType::ObjDump,
            registers: None,
            ..Default::default()
        };

        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::LABEL
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::INSTRUCTION
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::INSTRUCTION
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::DIRECTIVE
        );
    }

    #[test]
    fn assembly_peek() {
        let data = r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:"#;

        let mut lines = data.split('\n');
        let config = ParserConfig {
            architecture: Architecture::AArch64,
            file_type: FileType::Assembly,
            registers: None,
            ..Default::default()
        };

        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::LABEL
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::DIRECTIVE
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::INSTRUCTION
        );
        assert_eq!(
            pre_process_next(lines.next().unwrap(), &config),
            SyntaxKind::LOCAL_LABEL
        );
    }
}
