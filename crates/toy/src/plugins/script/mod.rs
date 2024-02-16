use std::{boxed::Box, collections::HashMap, fs::read_to_string, path::PathBuf, sync::Arc};

use oxc::{
  ast::{
    ast::{
      BindingIdentifier, Expression, FormalParameterKind, FunctionType, Modifiers,
      ObjectPropertyKind, Program, PropertyKind, StringLiteral,
    },
    AstBuilder, Visit, VisitMut,
  },
  codegen::{Codegen, CodegenOptions},
  span::{SourceType, Span},
};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::module::{Module, ModuleKind, ModuleMeta, ScriptModuleMeta},
  oxc::OxcProgram,
  plugin::{AnalyzeDepsHookParams, LoadHookParams, LoadHookResult, ParseHookParams, Plugin},
  resource::{
    resource::{Resource, ResourceKind, ResourceMap},
    resource_pot::{JsResourcePotMeta, ResourcePot, ResourcePotKind, ResourcePotMeta},
  },
  utils::{fulfill_root_prefix, stripe_root_prefix},
};

use self::{deps_visitor::DepsVisitor, esm_visitor::EsmVisitor, runtime_visitor::RuntimeVisitor};

mod deps_visitor;
mod esm_visitor;
mod runtime_visitor;

pub struct PluginScript {}

impl PluginScript {
  pub fn new() -> Self {
    Self {}
  }

  /// 构造模块 wrapper 函数
  ///
  /// ```js
  /// console.log('Hello World')
  /// ```
  ///
  /// ↓↓↓
  ///
  /// ```js
  /// function (module, exports, __toyRequire__, __toyDynamicRequire__) {
  ///   console.log('Hello World')
  /// }
  /// ```
  fn wrap_module<'a>(
    &self,
    ast_builder: &'a AstBuilder<'a>,
    program: Program<'a>,
  ) -> Expression<'a> {
    let fn_params_item = ["__toyModule__", "__toyRequire__", "__toyDynamicRequire__"].map(|name| {
      ast_builder.formal_parameter(
        Span::default(),
        ast_builder.binding_pattern(
          ast_builder
            .binding_pattern_identifier(BindingIdentifier::new(Span::default(), name.into())),
          None,
          false,
        ),
        None,
        false,
        ast_builder.new_vec(),
      )
    });

    // wrapper 函数的参数
    let mut fn_params = ast_builder.formal_parameters(
      Span::default(),
      FormalParameterKind::FormalParameter,
      ast_builder.new_vec(),
      None,
    );
    fn_params.items.extend(fn_params_item);

    // wrapper 函数的函数体
    let fn_body = ast_builder.function_body(
      Span::default(),
      ast_builder.new_vec(),
      ast_builder.copy(&program.body),
    );

    // 完整的 wrapper 函数
    ast_builder.function_expression(ast_builder.function(
      FunctionType::FunctionDeclaration,
      Span::default(),
      None,
      false,
      false,
      None,
      fn_params,
      Some(fn_body),
      None,
      None,
      Modifiers::empty(),
    ))
  }

  /// 把 `./js-runtime/module-system.js` 的代码解析成 ast
  fn get_module_system_ast(&self) -> OxcProgram {
    let module_system_str = include_str!("./js-runtime/module-system.js");

    OxcProgram::build(
      module_system_str.to_string(),
      SourceType::from_path("./js-runtime/module-system.js").unwrap(),
    )
  }
}

impl Plugin for PluginScript {
  fn name(&self) -> &str {
    "ToyPluginScript"
  }

  fn load(
    &self,
    params: &LoadHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<LoadHookResult>> {
    let module_kind = ModuleKind::from_file_path(&params.id);

    if module_kind.is_script() {
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
    if params.module_kind.is_script() {
      let source_type = SourceType::from_path(params.id.clone()).unwrap();
      let ast = OxcProgram::build(params.content.clone(), source_type);

      let module = Module::new(
        params.id.to_string(),
        params.module_kind.clone(),
        Some(ModuleMeta::Script(ScriptModuleMeta {
          code: params.content.clone(),
          ast,
        })),
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
    if let ModuleMeta::Script(meta) = &params.module.meta {
      // 访问 program，查找依赖项
      let program = &meta.ast.copy_program();
      let mut deps_visitor = DepsVisitor::new();
      deps_visitor.visit_program(program);

      // 把查找到的依赖项推到 params.deps 中
      params.deps.extend(deps_visitor.deps);
    }

    Ok(())
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if matches!(resource_pot.kind, ResourcePotKind::Js) {
      let module_graph = context.module_graph.write().unwrap();
      let mut modules_object_properties_vec = vec![];

      if module_graph.is_entry_module(&resource_pot.id) {
        // 入口 JS 模块
        // 遍历 resource_pot 里的每一个模块，用 wrapper 函数包裹模块
        let ast_builder = context.ast_builder.get_ast_builder();

        for module_id in resource_pot.module_ids.iter() {
          let module = module_graph.module(module_id).unwrap();
          let mut program = module.meta.as_script().ast.copy_program();
          let mut esm_visitor = EsmVisitor::new(ast_builder, module_id, &module_graph);

          esm_visitor.visit_program(&mut program);

          modules_object_properties_vec.push(ObjectPropertyKind::ObjectProperty(
            ast_builder.object_property(
              Span::default(),
              PropertyKind::Init,
              ast_builder.property_key_expression(ast_builder.literal_string_expression(
                StringLiteral::new(Span::default(), module.id.clone().into()),
              )),
              self.wrap_module(&ast_builder, program),
              None,
              false,
              false,
              false,
            ),
          ));
        }

        // 注入模块系统运行时
        let runtime_oxc_program = self.get_module_system_ast();
        let mut runtime_program = runtime_oxc_program.copy_program();

        let mut modules_object_properties = ast_builder.new_vec();
        modules_object_properties.extend(modules_object_properties_vec);
        let modules_object_expr =
          ast_builder.object_expression(Span::default(), modules_object_properties, None);
        let resource_id = resource_pot.id.clone();
        let entry_id_expr = ast_builder
          .literal_string_expression(StringLiteral::new(Span::default(), resource_id.into()));
        let mut runtime_visitor =
          RuntimeVisitor::new(&ast_builder, modules_object_expr, entry_id_expr);

        runtime_visitor.visit_program(&mut runtime_program);

        let source_len = resource_pot.module_ids.iter().fold(0, |acc, module_id| {
          let module = module_graph.module(module_id).unwrap();
          acc + module.meta.as_script().code.len()
        });

        let code = Codegen::<false>::new(source_len, CodegenOptions).build(&runtime_program);

        resource_pot.meta = ResourcePotMeta::Js(JsResourcePotMeta { ast: None, code });
      } else {
        // 动态加载的 JS 模块，注入模块注册运行时
        // TODO
      }
    }

    Ok(())
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourceMap>> {
    if let ResourcePotMeta::Js(js_resource_pot_meta) = &resource_pot.meta {
      let mut resource_map: ResourceMap = HashMap::new();
      let resource_id = resource_pot.id.clone();
      let name = stripe_root_prefix(&resource_id);

      resource_map.insert(
        resource_id.clone(),
        Resource {
          name,
          content: js_resource_pot_meta.code.clone(),
          resource_kind: ResourceKind::Js,
          resource_pot_id: resource_pot.id.clone(),
          emitted: false,
        },
      );

      resource_pot.resource_ids.push(resource_id);

      return Ok(Some(resource_map));
    }

    Ok(None)
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
            .to_string_lossy()
            .to_string(),
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
