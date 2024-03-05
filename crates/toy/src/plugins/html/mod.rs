use std::{collections::HashMap, fs::read_to_string, path::PathBuf, sync::Arc};
use swc_common::{BytePos, FileName, SourceFile};
use swc_html::{
  codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{parse_file_as_document, parser::ParserConfig},
};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::module::{HtmlModuleMeta, Module, ModuleKind, ModuleMeta},
  plugin::{LoadHookParams, LoadHookResult, ParseHookParams, Plugin},
  resource::{
    self,
    resource::{Resource, ResourceKind, ResourceMap},
    resource_pot::{HtmlResourcePotMeta, ResourcePot, ResourcePotKind, ResourcePotMeta},
  },
  utils::fulfill_root_prefix,
};

use self::{deps_visitor::DepsVisitor, resources_injector::ResourcesInjector};

mod deps_visitor;
mod resources_injector;

pub struct PluginHtml {}

impl PluginHtml {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginHtml {
  fn name(&self) -> &str {
    "ToyPluginHtml"
  }

  fn load(
    &self,
    params: &LoadHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<LoadHookResult>> {
    let module_kind = ModuleKind::from_file_path(&params.id);

    if module_kind.is_html() {
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
    params: &ParseHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<Module>> {
    if params.module_kind.is_html() {
      let source_file = SourceFile::new(
        FileName::Real(PathBuf::from(&params.id)),
        false,
        FileName::Real(PathBuf::from(&params.id)),
        params.content.clone(),
        BytePos(1),
      );

      let ast = parse_file_as_document(&source_file, ParserConfig::default(), &mut vec![]).unwrap();

      let module = Module::new(
        params.id.to_string(),
        params.module_kind.clone(),
        Some(ModuleMeta::Html(HtmlModuleMeta { ast })),
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
    if params.module.kind.is_html() {
      // 收集 html 的依赖
      // - script src
      // - link href
      let mut deps_visitor = DepsVisitor::new();
      let deps = deps_visitor.analyze_deps(&params.module.meta.as_html().ast);
      params.deps.extend(deps);
    }

    Ok(())
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if matches!(resource_pot.kind, ResourcePotKind::Html) {
      let module_graph = context.module_graph.read().unwrap();

      if resource_pot.module_ids.len() > 1 {
        return Err(CompilationError::GenericError(
          "Multiple html modules are not supported".to_string(),
        ));
      }

      let html_module = module_graph.module(&resource_pot.module_ids[0]).unwrap();
      resource_pot.meta = ResourcePotMeta::Html(HtmlResourcePotMeta {
        ast: html_module.meta.as_html().ast.clone(),
      })
    }

    Ok(())
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourceMap>> {
    if let ResourcePotMeta::Html(_) = &resource_pot.meta {
      let mut resource_map: ResourceMap = HashMap::new();
      let resource_id = resource_pot.id.clone();

      resource_map.insert(
        resource_id.clone(),
        Resource {
          name: resource_id.clone(),
          content: "".to_string(), // html 资源内容会在 write_resources 时才生成
          resource_kind: ResourceKind::Html,
          resource_pot_id: resource_id.clone(),
          emitted: false,
        },
      );

      resource_pot.resource_ids.push(resource_id);

      return Ok(Some(resource_map));
    }

    Ok(None)
  }

  fn write_resources(
    &self,
    resources: &mut crate::resource::resource::ResourceMap,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    let mut resource_pot_map = context.resource_pot_map.write().unwrap();
    let module_graph = context.module_graph.read().unwrap();
    let mut html_to_dep_resource_ids = HashMap::new();

    for (html_resource_id, html_resource) in resources.iter() {
      if matches!(html_resource.resource_kind, ResourceKind::Html) {
        let module_group_map = context.module_group_map.read().unwrap();
        let html_resource_pot = resource_pot_map
          .get(&html_resource.resource_pot_id)
          .unwrap();
        let module_group = module_group_map
          .get(&html_resource_pot.module_group_id)
          .unwrap();

        // 收集 html 依赖的资源列表
        for resource_pot_id in module_group.resource_pot_ids() {
          if resource_pot_id == &html_resource_pot.id {
            continue;
          }

          let dep_resource_pot = resource_pot_map.get(resource_pot_id).unwrap();

          html_to_dep_resource_ids.insert(
            html_resource_id.to_string(),
            dep_resource_pot.resource_ids.clone(),
          );
        }
      }
    }

    for (html_resource_id, dep_resource_ids) in html_to_dep_resource_ids {
      // 分类 js 和 css 资源
      let mut js_resources = vec![];
      let mut css_resources = vec![];

      for dep_resource_id in dep_resource_ids {
        let dep_resource = resources.get(&dep_resource_id).unwrap();

        match dep_resource.resource_kind {
          ResourceKind::Js => js_resources.push(dep_resource.name.clone()),
          ResourceKind::Css => css_resources.push(dep_resource.name.clone()),
          _ => {}
        }
      }

      let html_resource = resources.get_mut(&html_resource_id).unwrap();
      let html_resource_pot = resource_pot_map
        .get_mut(&html_resource.resource_pot_id)
        .unwrap();

      // 获取 html 的直接依赖
      let deps = module_graph
        .dependencies(&html_resource_pot.module_ids[0])
        .unwrap()
        .into_iter()
        .map(|(_, edge)| edge.source.clone())
        .collect::<Vec<String>>();

      // 注入 css 和 js 资源到 ast
      let mut resources_injector = ResourcesInjector::new(deps, css_resources, js_resources);
      let document = &mut html_resource_pot.meta.as_html_mut().ast;
      resources_injector.inject(document);

      // 根据 ast 生成最终代码
      let mut html_code = String::new();
      let html_writer =
        BasicHtmlWriter::new(&mut html_code, None, BasicHtmlWriterConfig::default());
      let mut html_gen = CodeGenerator::new(
        html_writer,
        CodegenConfig {
          minify: false,
          ..Default::default()
        },
      );

      html_gen.emit(document).unwrap();

      // 修改 html resource 的 content 字段，以让 resources 插件把内容输出到文件系统
      html_resource.content = html_code;
    }

    Ok(())
  }
}
