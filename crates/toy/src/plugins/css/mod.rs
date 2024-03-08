use std::{collections::HashMap, fs::read_to_string, sync::Arc};

use lightningcss::{
  printer::PrinterOptions,
  rules::{CssRule, CssRuleList},
  stylesheet::{ParserOptions, StyleSheet},
  traits::IntoOwned,
  visitor::Visit,
};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  lightningcss::LightningStyleSheet,
  module::module::{CssModuleMeta, Module, ModuleKind, ModuleMeta},
  plugin::{AnalyzeDepsHookParams, LoadHookParams, LoadHookResult, Plugin},
  resource::{
    resource::{Resource, ResourceKind, ResourceMap},
    resource_pot::{CssResourcePotMeta, ResourcePot, ResourcePotKind, ResourcePotMeta},
  },
  utils::fulfill_root_prefix,
};

use self::deps_visitor::DepsVisitor;

mod deps_visitor;

pub struct PluginCss {}

impl PluginCss {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginCss {
  fn name(&self) -> &str {
    "ToyPluginCss"
  }

  fn load(
    &self,
    params: &LoadHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<LoadHookResult>> {
    let module_kind = ModuleKind::from_file_path(&params.id);

    if module_kind.is_style() {
      let path = fulfill_root_prefix(&params.id, &context.config.root);

      let content = read_to_string(path).map_err(|err| CompilationError::LoadError {
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
    params: &crate::plugin::ParseHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<crate::module::module::Module>> {
    if params.module_kind.is_style() {
      let style_sheet = LightningStyleSheet::build(params.content.clone(), params.id.clone());

      let module = Module::new(
        params.id.to_string(),
        params.module_kind.clone(),
        Some(ModuleMeta::Css(CssModuleMeta { ast: style_sheet })),
      );

      return Ok(Some(module));
    }

    Ok(None)
  }

  fn analyze_deps(
    &self,
    params: &mut AnalyzeDepsHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if params.module.kind.is_style() {
      let mut deps_visitor = DepsVisitor::new();
      let ast = &mut params.module.meta.as_css_mut().ast;

      ast.with_style_sheet_mut(|style_sheet| {
        // 收集 css 依赖：@import
        style_sheet.visit(&mut deps_visitor).unwrap();

        // 删除 @import
        let _ = style_sheet
          .rules
          .0
          .iter()
          .filter(|rule| matches!(rule, CssRule::Import(_)));
      });

      params.deps = deps_visitor.deps;
    }

    Ok(())
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if matches!(resource_pot.kind, ResourcePotKind::Css) {
      let module_graph = context.module_graph.read().unwrap();

      let mut merged_style_sheet: StyleSheet<'_, '_> =
        StyleSheet::new(vec![], CssRuleList(vec![]), ParserOptions::default());

      for module_id in &resource_pot.module_ids {
        let module = module_graph.module(module_id).unwrap();
        let ast = &module.meta.as_css().ast;

        merged_style_sheet.sources.push(module_id.to_string());

        ast.with_style_sheet(|style_sheet| {
          merged_style_sheet
            .rules
            .0
            .extend(style_sheet.rules.clone().into_owned().0);
        });
      }

      let merged_css_code = merged_style_sheet
        .to_css(PrinterOptions::default())
        .unwrap()
        .code;

      resource_pot.meta = ResourcePotMeta::Css(CssResourcePotMeta {
        ast: None,
        code: merged_css_code,
      });
    }

    Ok(())
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourceMap>> {
    if let ResourcePotMeta::Css(css_resource_pot_meta) = &resource_pot.meta {
      let mut resource_map: ResourceMap = HashMap::new();
      let resource_id = resource_pot.id.clone();

      resource_map.insert(
        resource_id.clone(),
        Resource {
          name: resource_id.clone(),
          content: css_resource_pot_meta.code.clone(),
          resource_kind: ResourceKind::Css,
          resource_pot_id: resource_id.clone(),
          emitted: false,
        },
      );

      resource_pot.resource_ids.push(resource_id);

      return Ok(Some(resource_map));
    }

    Ok(None)
  }
}
