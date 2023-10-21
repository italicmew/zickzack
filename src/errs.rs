use std::fmt;

use actix_web::HttpResponse;
use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub enum EditorError {
    ContentNotFound,
    ReadFailure,
    SaveFailure,
    DirectoryCreationFailure,
    InvalidPath,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    description: String,
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EditorError::ContentNotFound => write!(f, "Content not found."),
            EditorError::ReadFailure => write!(f, "Failed to read content."),
            EditorError::SaveFailure => write!(f, "Failed to save content."),
            EditorError::DirectoryCreationFailure => write!(f, "Failed to create directories."),
            EditorError::InvalidPath => write!(f, "Invalid path."),
        }
    }
}

impl EditorError {
    fn to_error_response(&self) -> ErrorResponse {
        match *self {
            EditorError::ContentNotFound => ErrorResponse {
                error: "ContentNotFound".to_string(),
                description: "Content not found.".to_string(),
            },
            EditorError::ReadFailure => ErrorResponse {
                error: "ReadFailure".to_string(),
                description: "Failed to read content.".to_string(),
            },
            EditorError::SaveFailure => ErrorResponse {
                error: "SaveFailure".to_string(),
                description: "Unable to save text.".to_string(),
            },
            EditorError::DirectoryCreationFailure => ErrorResponse {
                error: "DirectoryCreationFailure".to_string(),
                description: "Failed to create new directory.".to_string(),
            },
            EditorError::InvalidPath => ErrorResponse {
                error: "InvalidPath".to_string(),
                description: "Invalid Path.".to_string(),
            },
        }
    }
}


impl actix_web::error::ResponseError for EditorError {
    fn error_response(&self) -> HttpResponse {
        let error_response = self.to_error_response();
        match *self {
            EditorError::ContentNotFound => HttpResponse::NotFound().json(error_response),
            EditorError::ReadFailure => HttpResponse::InternalServerError().json(error_response),
            EditorError::SaveFailure => HttpResponse::InternalServerError().json(error_response),
            EditorError::DirectoryCreationFailure => HttpResponse::InternalServerError().json(error_response),
            EditorError::InvalidPath => HttpResponse::NotFound().json(error_response),
        }
    }
}