use std::{fs::read_to_string, sync::Arc};

use oxc::{allocator::Allocator, ast::Visit, parser::Parser, span::SourceType};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::module::{Module, ModuleKind, ModuleMeta, ScriptAst, ScriptModuleMeta},
  plugin::{LoadHookParams, LoadHookResult, ParseHookParams, Plugin},
};

use self::deps_visitor::DepsVisitor;

mod deps_visitor;

pub struct PluginScript {}

impl PluginScript {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginScript {
  fn name(&self) -> &str {
    "ToyPluginScript"
  }

  fn load(
    &self,
    params: &LoadHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<LoadHookResult>> {
    let module_kind = ModuleKind::from_file_path(&params.id);

    if module_kind.is_script() {
      let content = read_to_string(params.id).map_err(|err| CompilationError::LoadError {
        id: params.id.to_string(),
        source: Some(Box::new(err)),
      })?;

      return Ok(Some(LoadHookResult {
        content,
        module_kind,
      }));
    }

    Ok(None)
  }

  fn parse(
    &self,
    params: &ParseHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<Module>> {
    if params.module_kind.is_script() {
      let source_type = SourceType::from_path(params.id).unwrap();
      let ast: ScriptAst = ScriptAst::build(params.content.clone(), source_type);

      let module = Module::new(
        params.id.to_string(),
        params.module_kind.clone(),
        Some(ModuleMeta::Script(ScriptModuleMeta { ast })),
      );

      return Ok(Some(module));
    }

    Ok(None)
  }

  fn analyze_deps(
    &self,
    params: &mut crate::plugin::AnalyzeDepsHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if let ModuleMeta::Script(meta) = &params.module.meta {
      meta.ast.with_program(|program| {
        // 访问 program，查找依赖项
        let mut deps_visitor = DepsVisitor::new();
        deps_visitor.visit_program(&program.0);

        // 把查找到的依赖项推到 params.deps 中
        params.deps.extend(deps_visitor.deps);
      })
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, fs};

  use crate::config::Config;

  use super::*;

  #[test]
  fn test_load() {
    let context = CompilationContext::new(Config::default(), vec![]);
    let plugin_script = PluginScript::new();

    let res = plugin_script
      .load(
        &LoadHookParams {
          id: fs::canonicalize("../../fixtures/basic/index.js")
            .unwrap()
            .to_str()
            .unwrap(),
          query: HashMap::new(),
        },
        &Arc::new(context),
      )
      .unwrap()
      .unwrap();

    assert!(!res.content.is_empty());
    assert_eq!(res.module_kind, ModuleKind::Js);
  }
}
