use std::{any::Any, collections::HashMap, sync::Arc};

use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module::{Module, ModuleKind},
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupMap},
    ResolveKind,
  },
  resource::{resource::ResourceMap, resource_pot::ResourcePot},
};

pub mod plugin_container;

pub const DEFAULT_PRIORITY: i32 = 100;

#[derive(Debug)]
pub struct ResolveHookParams {
  pub source: String,
  pub importer: Option<String>,
  pub kind: ResolveKind,
}

#[derive(Debug)]
pub struct ResolveHookResult {
  pub id: String,
  pub query: HashMap<String, String>,
  pub external: bool,
}

#[derive(Debug)]
pub struct LoadHookParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LoadHookResult {
  pub content: String,
  pub module_kind: ModuleKind,
}

#[derive(Debug)]
pub struct TransformHookParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
  pub content: String,
  pub module_kind: ModuleKind,
}

#[derive(Debug)]
pub struct TransformHookResult {
  pub content: String,
  pub module_kind: Option<ModuleKind>,
  pub source_map: Option<String>,
}

#[derive(Debug)]
pub struct ParseHookParams<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
  pub content: String,
  pub module_kind: ModuleKind,
}

pub struct AnalyzeDepsHookParams<'a> {
  pub module: &'a Module,
  pub deps: Vec<AnalyzeDep>,
}

#[derive(Debug)]
pub struct AnalyzeDep {
  pub source: String,
  pub resolve_kind: ResolveKind,
}

pub trait Plugin: Any + Send + Sync {
  fn name(&self) -> &str;

  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY
  }

  fn config(&self, _config: &mut Config) -> Result<()> {
    Ok(())
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<()> {
    Ok(())
  }

  fn resolve(
    &self,
    _params: &ResolveHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResolveHookResult>> {
    Ok(None)
  }

  fn load(
    &self,
    _params: &LoadHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<LoadHookResult>> {
    Ok(None)
  }

  fn transform(
    &self,
    _params: &TransformHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<TransformHookResult>> {
    Ok(None)
  }

  fn parse(
    &self,
    _params: &ParseHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<Module>> {
    Ok(None)
  }

  fn analyze_deps(
    &self,
    _params: &mut AnalyzeDepsHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn build_end(&self, _context: &Arc<CompilationContext>) -> Result<()> {
    Ok(())
  }

  fn generate_start(&self, _context: &Arc<CompilationContext>) -> Result<()> {
    Ok(())
  }

  fn analyze_module_graph(
    &self,
    _module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ModuleGroupMap>> {
    Ok(None)
  }

  fn analyze_module_group(
    &self,
    _module_group: &mut ModuleGroup,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<Vec<ResourcePot>>> {
    Ok(None)
  }

  fn render_resource_pot(
    &self,
    _resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn generate_resources(
    &self,
    _resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourceMap>> {
    Ok(None)
  }

  fn write_resources(
    &self,
    _resources: &mut ResourceMap,
    _context: &Arc<CompilationContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<()> {
    Ok(())
  }
}
