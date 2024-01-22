use std::{any::Any, collections::HashMap, sync::Arc};

use crate::{context::CompilationContext, error::Result, module::ModuleKind};

pub struct TransformParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
  pub content: String,
  pub module_kind: ModuleKind,
}

pub struct TransformResult {
  pub content: String,
  pub source_map_chain: Vec<String>,
  pub module_kind: ModuleKind,
}

pub fn transform(
  params: TransformParams,
  context: &Arc<CompilationContext>,
) -> Result<TransformResult> {
  Ok(TransformResult {
    content: params.content,
    source_map_chain: vec![],
    module_kind: params.module_kind,
  })
}
