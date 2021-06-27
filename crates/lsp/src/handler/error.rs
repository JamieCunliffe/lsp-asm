use lsp_server::ResponseError;

pub(crate) enum ErrorCode {
    NoRoot,
    TokenNotFound,
    InvalidPosition,
    CastFailed,
    FileNotFound,
}

pub(crate) fn lsp_error_map(error: ErrorCode) -> ResponseError {
    match error {
        ErrorCode::NoRoot => ResponseError {
            code: 1,
            message: String::from("Syntax tree error, no root node found"),
            data: None,
        },
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
    }
}
