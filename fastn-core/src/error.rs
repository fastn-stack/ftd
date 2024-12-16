#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[error("ZipError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("SerdeJsonError: {}", _0)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("FTDError: {}", _0)]
    FTDError(#[from] ftd::ftd2021::p1::Error),

    #[error("FTDP1Error: {}", _0)]
    FTDP1Error(#[from] ftd_p1::Error),

    #[error("FTDInterpolationError: {}", _0)]
    FTDInterpolationError(#[from] fastn_expr::interpolator::InterpolationError),

    #[error("FTDAstError: {}", _0)]
    FTDAstError(#[from] ftd_ast::Error),

    #[error("FTDExecError: {}", _0)]
    FTDExecError(#[from] ftd::executor::Error),

    #[error("FTDInterpreterError: {}", _0)]
    FTDInterpreterError(#[from] ftd::interpreter::Error),

    #[error("FTDHtmlError: {}", _0)]
    FTDHtmlError(#[from] ftd::html::Error),

    #[error("IgnoreError: {}", _0)]
    IgnoreError(#[from] ignore::Error),

    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),

    #[error("StripPrefixError: {}", _0)]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("SitemapParseError: {}", _0)]
    SitemapParseError(#[from] fastn_core::sitemap::ParseError),

    #[error("URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),

    #[error("UTF8Error: {}", _0)]
    UTF8Error(#[from] std::string::FromUtf8Error),

    #[error("ParseIntError: {}", _0)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("ParseFloatError: {}", _0)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("ParseBoolError: {}", _0)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("APIResponseError: {}", _0)]
    APIResponseError(String),

    #[error("NotFoundError: {}", _0)]
    NotFound(String),

    #[error("FastnIoError: {io_error}, path: {path}")]
    FastnIoError {
        io_error: std::io::Error,
        path: String,
    },

    #[error("PackageError: {message}")]
    PackageError { message: String },

    #[error("UsageError: {message}")]
    UsageError { message: String },

    #[error("UpdateError: {message}")]
    UpdateError { message: String },

    #[error("GenericError: {}", _0)]
    GenericError(String),

    #[error("GroupNotFound: id: {id}, {message}")]
    GroupNotFound { id: String, message: String },

    #[error("CRAboutNotFound CR#{cr_number}: {message}")]
    CRAboutNotFound { message: String, cr_number: usize },

    #[error("QueryPayloadError: {}", _0)]
    QueryPayloadError(#[from] actix_web::error::QueryPayloadError),

    #[error("TokioMPSCError2: {}", _0)]
    TokioMPSCError2(#[from] tokio::sync::mpsc::error::SendError<usize>),

    #[error("MissingEnvironmentVariableError: {}", _0)]
    EnvironmentVariableError(#[from] std::env::VarError),

    #[error("BoolEnvironmentError: {}", _0)]
    BoolEnvironmentError(#[from] fastn_ds::BoolEnvironmentError),

    #[error("DatabaseError: {message}")]
    DatabaseError { message: String },

    #[error("ds::ReadError: {}", _0)]
    DSReadError(#[from] fastn_ds::ReadError),

    #[error("ds::ReadStringError: {}", _0)]
    DSReadStringError(#[from] fastn_ds::ReadStringError),

    #[error("ds::WriteError: {}", _0)]
    DSWriteError(#[from] fastn_ds::WriteError),

    #[error("ds::RemoveError: {}", _0)]
    DSRemoveError(#[from] fastn_ds::RemoveError),

    #[error("ds::RenameError: {}", _0)]
    DSRenameError(#[from] fastn_ds::RenameError),

    #[error("ds::CreatePoolError: {}", _0)]
    CreatePool(#[from] fastn_ds::CreatePoolError),

    #[error("pool error: {0}")]
    PoolError(#[from] deadpool::managed::PoolError<tokio_postgres::Error>),

    #[error("ds::HttpError: {}", _0)]
    DSHttpError(#[from] fastn_ds::HttpError),

    #[error("AssertError: {message}")]
    AssertError { message: String },

    #[error("config_temp::Error: {}", _0)]
    ConfigTempError(#[from] fastn_core::config_temp::Error),

    #[error("FormError: {:?}", _0)]
    FormError(std::collections::HashMap<String, String>),

    #[error("SSRError: {:?}", _0)]
    SSRError(#[from] fastn_js::SSRError),

    #[error("MigrationError: {0}")]
    MigrationError(#[from] fastn_core::migrations::MigrationError),

    #[error("UnknownHandler")]
    UnknownHandler,
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl Error {
    pub fn generic<T: AsRef<str> + ToString>(error: T) -> Self {
        Self::GenericError(error.to_string())
    }

    pub fn generic_err<T: AsRef<str> + ToString, O>(error: T) -> fastn_core::Result<O> {
        Err(Self::generic(error))
    }

    pub fn to_html(&self) -> actix_web::HttpResponse {
        // TODO: hate this error type, have no idea how to handle things properly at this stage now
        //       we should remove this type and write more precise error types
        match self {
            Error::FormError(errors) => {
                tracing::info!("form error: {:?}", errors);
                actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .json(serde_json::json!({"errors": errors}))
            }
            Error::NotFound(message) => {
                tracing::info!("not found: {:?}", message);
                actix_web::HttpResponse::NotFound().body(message.to_string())
            }
            Error::DSReadError(fastn_ds::ReadError::NotFound(f)) => {
                tracing::info!("ds read error, not found: {f}");
                actix_web::HttpResponse::NotFound().body("page not found: {f}")
            }
            _ => {
                tracing::error!("error: {:?}", self);
                actix_web::HttpResponse::InternalServerError()
                    .body(format!("internal server error: {self:?}"))
            }
        }
    }
}
