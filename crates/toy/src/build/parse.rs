use std::{any::Any, collections::HashMap, sync::Arc};

use crate::{
  context::CompilationContext,
  error::Result,
  module::{Module, ModuleKind},
};

pub struct ParseParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
  pub content: String,
  pub module_kind: ModuleKind,
}

pub struct ParseResult {
  pub module: Module,
}

pub fn parse(params: ParseParams, context: &Arc<CompilationContext>) -> Result<ParseResult> {
  Ok(ParseResult {
    module: Module::new(params.id.to_string(), params.module_kind),
  })
}
