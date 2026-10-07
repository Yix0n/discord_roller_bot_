use serenity::all::{CreateEmbedFooter};
use serenity::builder::CreateEmbed;
use uuid::Error as UUIDError;

#[derive(Debug)]
pub struct ErrorContext {
    pub module: String,
    pub error_type: ProgramError,
    pub message: String,
}

#[derive(Debug)]
pub enum ProgramError {
    NotationError,
    SubcommandParseError,
    FormatError,
    DatabaseError,
    SerdeError,
    CommandError,
    ParseError,
    ThreadError,
    NotFound,
}

impl ErrorContext {
    pub fn new(module: &str, error_type: ProgramError, message: &str) -> Self {
        Self {
            module: String::from(module),
            error_type,
            message: String::from(message),
        }
    }

    pub fn to_embed(&self) -> CreateEmbed {
        let embed = CreateEmbed::new()
            .title("Error")
            .footer(CreateEmbedFooter::new(format!("Module: {}", self.module)))
            .description(format!("{}", self.message));

        embed
    }
}

impl From<rusqlite::Error> for ErrorContext {
    fn from(error: rusqlite::Error) -> Self {
        Self {
            module: "database".to_string(),
            error_type: ProgramError::DatabaseError,
            message: error.to_string(),
        }
    }
}

impl From<UUIDError> for ErrorContext {
    fn from(error: UUIDError) -> Self {
        Self {
            module: "parser".to_string(),
            error_type: ProgramError::ParseError,
            message: error.to_string(),
        }
    }
}