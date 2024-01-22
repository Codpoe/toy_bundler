use crate::{
  context::CompilationContext,
  error::Result,
  plugin::{Plugin, ResolveHookParams, ResolveHookResult},
};

pub struct PluginResolve {}

impl PluginResolve {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginResolve {
  fn name(&self) -> &str {
    "ToyPluginResolve"
  }

  fn resolve(
    &self,
    params: &ResolveHookParams,
    context: &std::sync::Arc<CompilationContext>,
  ) -> Result<Option<ResolveHookResult>> {
    println!("resolve {params:?}");
    Ok(None)
  }
}
