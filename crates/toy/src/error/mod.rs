use std::{error::Error, result::Result as StdResult};
use thiserror::Error;

/// 错误枚举
#[derive(Error, Debug)]
pub enum CompilationError {
  /// 一般错误
  #[error("{0}")]
  GenericError(String),

  // resolve
  #[error("Hook `resolve` execute failed. Error: {source:?}")]
  ResolveError {
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },
}

pub type Result<T> = StdResult<T, CompilationError>;
