use lsp_server::ResponseError;
use lsp_types::Url;
use serde::Serialize;

pub(crate) enum ErrorCode {
    TokenNotFound,
    InvalidPosition,
    CastFailed,
    FileNotFound,
    InvalidVersion(Url),
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
    }
}
