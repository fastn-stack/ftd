extern crate self as fastn_package;

mod initialize;
pub mod initializer;
pub(crate) mod sqlite;

pub use initialize::initialize;

static FTD_CACHE: tokio::sync::OnceCell<
    tokio::sync::RwLock<std::collections::HashMap<String, ftd::ast::AST>>,
> = tokio::sync::OnceCell::const_new();
