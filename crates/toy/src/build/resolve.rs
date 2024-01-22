use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{context::CompilationContext, error::Result, module::ResolveKind};

pub struct ResolveParams {
  pub source: String,
  pub importer: Option<String>,
  pub kind: ResolveKind,
}

pub struct ResolveResult {
  pub id: String,
  pub query: HashMap<String, String>,
}

pub fn resolve(params: &ResolveParams, context: &Arc<CompilationContext>) -> Result<ResolveResult> {
  Ok(ResolveResult {
    id: "".to_string(),
    query: HashMap::new(),
  })
}
