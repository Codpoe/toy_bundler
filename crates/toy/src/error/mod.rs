use std::result::Result as StdResult;
use thiserror::Error;

/// 错误枚举
#[derive(Error, Debug)]
pub enum CompilationError {
  /// 一般错误
  #[error("{0}")]
  GenericError(String),
}

pub type Result<T> = StdResult<T, CompilationError>;
