use std::{collections::HashMap, sync::Arc};

use crate::{context::CompilationContext, error::Result, module::ModuleKind};

pub struct LoadParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
}

pub struct LoadResult {
  pub content: String,
  pub module_kind: ModuleKind,
}

pub fn load(params: LoadParams, context: &Arc<CompilationContext>) -> Result<LoadResult> {
  Ok(LoadResult {
    content: "".to_string(),
    module_kind: ModuleKind::Js,
  })
}
