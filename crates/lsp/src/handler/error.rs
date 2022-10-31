use lsp_server::ResponseError;
use lsp_types::Url;
use serde::Serialize;

pub(crate) enum ErrorCode {
    TokenNotFound,
    InvalidPosition,
    CastFailed,
    FileNotFound,
    InvalidVersion(Url),
    MCAFailed(String),
    MissingParentNode,
    InvalidToken(String),
}

#[derive(Serialize)]
pub struct ResyncFile {
    uri: Url,
}

pub(crate) fn lsp_error_map(error: ErrorCode) -> ResponseError {
    match error {
        ErrorCode::TokenNotFound => ResponseError {
            code: 2,
            message: String::from("Token not found for position in syntax tree"),
            data: None,
        },
        ErrorCode::InvalidPosition => ResponseError {
            code: 3,
            message: String::from("Invalid position requested"),
            data: None,
        },
        ErrorCode::CastFailed => ResponseError {
            code: 4,
            message: String::from("Failed to cast syntax token"),
            data: None,
        },
        ErrorCode::FileNotFound => ResponseError {
            code: 5,
            message: String::from("The file specified in the request is not known"),
            data: None,
        },
        ErrorCode::InvalidVersion(uri) => ResponseError {
            code: 6,
            message: String::from("Incorrect version transition"),
            data: serde_json::to_value(ResyncFile { uri }).ok(),
        },
        ErrorCode::MCAFailed(reason) => ResponseError {
            code: 7,
            message: format!("Failed to run llvm-mca due to error: {reason}"),
            data: None,
        },
        ErrorCode::MissingParentNode => ResponseError {
            code: 8,
            message: String::from("No parent node attached"),
            data: None,
        },
        ErrorCode::InvalidToken(message) => ResponseError {
            code: 9,
            message,
            data: None,
        },
    }
}
