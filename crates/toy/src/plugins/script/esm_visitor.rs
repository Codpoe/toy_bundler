use std::{collections::HashMap, path::Path};

use oxc::{
  ast::{
    ast::{
      Argument, BindingIdentifier, BindingPatternKind, Declaration, ExportDefaultDeclarationKind,
      Expression, IdentifierName, IdentifierReference, ImportDeclarationSpecifier,
      ImportOrExportKind, Modifiers, ModuleDeclaration, ObjectPropertyKind, Program, PropertyKind,
      Statement, StringLiteral, VariableDeclarationKind,
    },
    AstBuilder, VisitMut,
  },
  span::Span,
  syntax::operator::AssignmentOperator,
};

use crate::module::module_graph::ModuleGraph;

struct ToyImport {
  source: String,
  kv: HashMap<String, String>,
}

struct ToyExport {
  spread: bool,
  kv: HashMap<String, String>,
}

/// 把 `esm` 模块转换成像 `commonjs` 的模块形式
///
/// ```js
/// import foo1, { foo2 } from './foo';
/// export const foo3 = 1;
/// export default
///
/// export function greet() {}
/// ```
///
/// ↓↓↓
///
/// ```js
/// const { default: foo1, foo2 } = __toyRequire__('./foo');
/// const foo3 = 1;
/// module.exports = { foo3 }
/// ```
pub struct EsmVisitor<'a> {
  ast_builder: &'a AstBuilder<'a>,
  module_id: &'a str,
  js_var_index: usize,
  imports: Vec<ToyImport>,
  exports: Vec<ToyExport>,
  dep_source_to_module_id: HashMap<String, String>,
}

impl<'a> EsmVisitor<'a> {
  pub fn new(
    ast_builder: &'a AstBuilder<'a>,
    module_id: &'a str,
    module_graph: &ModuleGraph,
  ) -> Self {
    let mut dep_source_to_module_id = HashMap::new();

    for (dep_id, edge) in module_graph.dependencies(module_id).unwrap() {
      dep_source_to_module_id.insert(edge.source, dep_id);
    }

    Self {
      ast_builder,
      module_id,
      js_var_index: 0,
      imports: vec![],
      exports: vec![],
      dep_source_to_module_id,
    }
  }

  fn id_to_js_var(&mut self, id: &str) -> String {
    let path = Path::new(id);
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();

    let js_var: String = name
      .chars()
      .map(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => c,
        _ => '_',
      })
      .collect();

    self.js_var_index += 1;

    "_".to_string() + js_var.as_str() + "$toy" + &self.js_var_index.to_string()
  }

  fn match_module_decl(
    &mut self,
    module_decl: &mut ModuleDeclaration<'a>,
  ) -> Option<Statement<'a>> {
    match module_decl {
      ModuleDeclaration::ImportDeclaration(import_decl) => {
        // 忽略类型导入：import type { Foo } from 'mod'
        if matches!(import_decl.import_kind, ImportOrExportKind::Type) {
          return None;
        }

        // 忽略样式 import
        if is_style_import(&import_decl.source) {
          return None;
        }

        let mut toy_import = ToyImport {
          source: import_decl.source.value.to_string(),
          kv: HashMap::new(),
        };

        // 遍历导入的 specifiers，构建 import key-value
        if let Some(specifiers) = &import_decl.specifiers {
          for specifier in specifiers {
            match specifier {
              ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) => {
                toy_import.kv.insert(
                  "default".to_string(),
                  default_specifier.local.name.to_string(),
                );
              }
              ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                // 忽略类型导入：import { type Foo } from 'mod'
                if matches!(specifier.import_kind, ImportOrExportKind::Type) {
                  continue;
                }

                toy_import.kv.insert(
                  specifier.imported.name().to_string(),
                  specifier.local.name.to_string(),
                );
              }
              ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_specifier) => {
                toy_import
                  .kv
                  .insert("*".to_string(), namespace_specifier.local.name.to_string());
              }
            }
          }
        }

        self.imports.push(toy_import);

        None
      }
      ModuleDeclaration::ExportDefaultDeclaration(export_decl) => {
        let mut ret_stmt = None;

        // export default 可以是变量、函数、类、表达式
        match &mut export_decl.declaration {
          // 函数
          ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            // 如果 export default 有 identifier，则使用 identifier 作为 export name，
            // 例如：export default function Foo() {}
            //
            // 否则使用模块 id 作为 export name
            // 例如：export default function() {}
            let export_name = if let Some(id) = &func.id {
              id.name.to_string()
            } else {
              self.id_to_js_var(self.module_id)
            };

            self.exports.push(ToyExport {
              spread: false,
              kv: HashMap::from([("default".to_string(), export_name)]),
            });
          }
          // 类
          ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let export_name = if let Some(id) = &class.id {
              id.name.to_string()
            } else {
              self.id_to_js_var(self.module_id)
            };

            self.exports.push(ToyExport {
              spread: false,
              kv: HashMap::from([("default".to_string(), export_name)]),
            });
          }
          // 表达式
          ExportDefaultDeclarationKind::Expression(expr) => match &expr {
            // 变量。例如：
            // const foo = 1
            // export default foo
            Expression::Identifier(id_ref) => {
              self.exports.push(ToyExport {
                spread: false,
                kv: HashMap::from([("default".to_string(), id_ref.name.to_string())]),
              });
            }
            // 其他表达式。例如：
            // export default 42
            _ => {
              let expr = self.ast_builder.move_expression(expr);

              let var_decl = self.ast_builder.variable_declaration(
                Span::default(),
                VariableDeclarationKind::Const,
                self.ast_builder.new_vec_single(
                  self.ast_builder.variable_declarator(
                    Span::default(),
                    VariableDeclarationKind::Const,
                    self.ast_builder.binding_pattern(
                      self
                        .ast_builder
                        .binding_pattern_identifier(BindingIdentifier::new(
                          Span::default(),
                          self.id_to_js_var(self.module_id).into(),
                        )),
                      None,
                      false,
                    ),
                    Some(expr),
                    false,
                  ),
                ),
                Modifiers::empty(),
              );

              ret_stmt = Some(Statement::Declaration(Declaration::VariableDeclaration(
                var_decl,
              )));
            }
          },
          _ => {}
        };

        ret_stmt
      }
      ModuleDeclaration::ExportNamedDeclaration(export_decl) => {
        // 忽略类型导出：export type { Foo }
        if matches!(export_decl.export_kind, ImportOrExportKind::Type) {
          return None;
        }

        let mut ret_stmt = None;
        let mut toy_import = None;
        let mut toy_export = ToyExport {
          spread: false,
          kv: HashMap::new(),
        };

        // 如果是 reexport，则新增 toy_import，
        // 例如：export { foo } from 'mod'
        if let Some(source) = &export_decl.source {
          // 忽略样式 import
          if is_style_import(source) {
            return None;
          }

          toy_import = Some(ToyImport {
            source: source.value.to_string(),
            kv: HashMap::new(),
          });
        }

        for specifier in &export_decl.specifiers {
          // 忽略类型导出：export { type Foo }
          if matches!(specifier.export_kind, ImportOrExportKind::Type) {
            continue;
          }

          let local = self.id_to_js_var(specifier.local.name().as_str());

          toy_import
            .as_mut()
            .unwrap()
            .kv
            .insert(specifier.local.name().to_string(), local.clone());

          toy_export
            .kv
            .insert(specifier.exported.name().to_string(), local.clone());
        }

        if let Some(decl) = &export_decl.declaration {
          match decl {
            // 函数
            // export function foo() {}
            Declaration::FunctionDeclaration(func) => {
              if let Some(id) = &func.id {
                toy_export
                  .kv
                  .insert(id.name.to_string(), id.name.to_string());
              }
            }
            // 类
            // export class Foo {}
            Declaration::ClassDeclaration(class) => {
              if let Some(id) = &class.id {
                toy_export
                  .kv
                  .insert(id.name.to_string(), id.name.to_string());
              }
            }
            // 变量
            // export const foo = 1;
            Declaration::VariableDeclaration(var_decl) => {
              for declarator in &var_decl.declarations {
                match &declarator.id.kind {
                  BindingPatternKind::BindingIdentifier(id) => {
                    toy_export
                      .kv
                      .insert(id.name.to_string(), id.name.to_string());
                  }
                  BindingPatternKind::ObjectPattern(obj_pattern) => {
                    for property in &obj_pattern.properties {
                      if let BindingPatternKind::BindingIdentifier(id) = &property.value.kind {
                        toy_export
                          .kv
                          .insert(id.name.to_string(), id.name.to_string());
                      }
                    }
                  }
                  BindingPatternKind::ArrayPattern(array_pattern) => {
                    for element in &array_pattern.elements {
                      if let Some(element) = element {
                        if let BindingPatternKind::BindingIdentifier(id) = &element.kind {
                          toy_export
                            .kv
                            .insert(id.name.to_string(), id.name.to_string());
                        }
                      }
                    }
                  }
                  _ => {}
                }
              }
            }
            _ => {}
          };

          ret_stmt = Some(Statement::Declaration(self.ast_builder.copy(decl)));
        }

        if let Some(toy_import) = toy_import {
          self.imports.push(toy_import);
        }
        self.exports.push(toy_export);

        ret_stmt
      }
      ModuleDeclaration::ExportAllDeclaration(export_decl) => {
        // 忽略类型导出
        if matches!(export_decl.export_kind, ImportOrExportKind::Type) {
          return None;
        }

        // 忽略样式 import
        if is_style_import(&export_decl.source) {
          return None;
        }

        let source = export_decl.source.value.to_string();
        let local = self.id_to_js_var(&source);
        let toy_import = ToyImport {
          source,
          kv: HashMap::from([("*".to_string(), local.clone())]),
        };
        let mut toy_export = ToyExport {
          spread: false,
          kv: HashMap::new(),
        };

        if let Some(export_name) = &export_decl.exported {
          toy_export
            .kv
            .insert(export_name.name().to_string(), local.clone());
        } else {
          toy_export.spread = true;
          toy_export.kv.insert(local.clone(), local.clone());
        }

        self.imports.push(toy_import);
        self.exports.push(toy_export);

        None
      }
      _ => None,
    }
  }

  fn build_toy_import_stmt(&self, import: &ToyImport) -> Statement<'a> {
    let call_expr = self.ast_builder.call_expression(
      Span::default(),
      self
        .ast_builder
        .identifier_reference_expression(IdentifierReference::new(
          Span::default(),
          "__toyRequire__".into(),
        )),
      self.ast_builder.new_vec_single(Argument::Expression(
        self
          .ast_builder
          .literal_string_expression(StringLiteral::new(
            Span::default(),
            self
              .dep_source_to_module_id
              .get(&import.source)
              .unwrap()
              .to_string()
              .into(),
          )),
      )),
      false,
      None,
    );

    let mut properties = self.ast_builder.new_vec_with_capacity(import.kv.len());

    for (key, value) in &import.kv {
      properties.push(
        self.ast_builder.binding_property(
          Span::default(),
          self
            .ast_builder
            .property_key_identifier(IdentifierName::new(Span::default(), key.to_string().into())),
          self.ast_builder.binding_pattern(
            self
              .ast_builder
              .binding_pattern_identifier(BindingIdentifier::new(
                Span::default(),
                value.to_string().into(),
              )),
            None,
            false,
          ),
          key == value,
          false,
        ),
      );
    }

    let var_decl = self.ast_builder.variable_declaration(
      Span::default(),
      VariableDeclarationKind::Const,
      self.ast_builder.new_vec_single(
        self.ast_builder.variable_declarator(
          Span::default(),
          VariableDeclarationKind::Const,
          self.ast_builder.binding_pattern(
            self
              .ast_builder
              .object_pattern(Span::default(), properties, None),
            None,
            false,
          ),
          Some(call_expr),
          false,
        ),
      ),
      Modifiers::empty(),
    );

    Statement::Declaration(Declaration::VariableDeclaration(var_decl))
  }

  fn build_toy_export_stmt(&self, exports: &Vec<ToyExport>) -> Statement<'a> {
    let mut properties = self.ast_builder.new_vec();

    for export in exports {
      for (key, value) in &export.kv {
        let export_name =
          self
            .ast_builder
            .identifier_reference_expression(IdentifierReference::new(
              Span::default(),
              value.to_string().into(),
            ));

        if export.spread {
          properties.push(ObjectPropertyKind::SpreadProperty(
            self
              .ast_builder
              .spread_element(Span::default(), export_name),
          ));
        } else {
          properties.push(ObjectPropertyKind::ObjectProperty(
            self.ast_builder.object_property(
              Span::default(),
              PropertyKind::Init,
              self
                .ast_builder
                .property_key_identifier(IdentifierName::new(
                  Span::default(),
                  key.to_string().into(),
                )),
              export_name,
              None,
              false,
              key == value,
              false,
            ),
          ));
        }
      }
    }

    let left = self.ast_builder.simple_assignment_target_member_expression(
      self.ast_builder.static_member(
        Span::default(),
        self
          .ast_builder
          .identifier_reference_expression(IdentifierReference::new(
            Span::default(),
            "__toyModule__".into(),
          )),
        IdentifierName::new(Span::default(), "exports".into()),
        false,
      ),
    );

    let right = self
      .ast_builder
      .object_expression(Span::default(), properties, None);

    self.ast_builder.expression_statement(
      Span::default(),
      self.ast_builder.assignment_expression(
        Span::default(),
        AssignmentOperator::Assign,
        left,
        right,
      ),
    )
  }
}

impl<'a> VisitMut<'a> for EsmVisitor<'a> {
  fn visit_program(&mut self, program: &mut Program<'a>) {
    for stmt in program.body.iter_mut() {
      match stmt {
        Statement::ModuleDeclaration(module_decl) => {
          // 如果返回了新的语句，则替换原来的语句，
          // 否则删除原来的语句
          if let Some(new_stmt) = self.match_module_decl(module_decl) {
            *stmt = new_stmt;
          } else {
            // *stmt = Statement::Empty(AstKind::Empty);
            self.ast_builder.move_statement(stmt);
          }
        }
        _ => (),
      };
    }

    for toy_import in &self.imports {
      let toy_import_stmt = self.build_toy_import_stmt(toy_import);
      program.body.insert(0, toy_import_stmt);
    }

    if !self.exports.is_empty() {
      let toy_export_stmt = self.build_toy_export_stmt(&self.exports);
      program.body.push(toy_export_stmt);
    }
  }
}

fn is_style_import(source: &StringLiteral) -> bool {
  source.value.to_string().ends_with(".css")
}
