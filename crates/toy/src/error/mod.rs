use std::{error::Error, result::Result as StdResult};
use thiserror::Error;

/// 错误枚举
#[derive(Error, Debug)]
pub enum CompilationError {
  /// 一般错误
  #[error("{0}")]
  GenericError(String),

  // resolve
  #[error("Can not resolve `{src}` from `{base}`.\nError: {source:?}")]
  ResolveError {
    src: String,
    base: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  // resolve
  #[error("Load `{id}` failed.\nError: {source:?}")]
  LoadError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },
}

pub type Result<T> = StdResult<T, CompilationError>;
