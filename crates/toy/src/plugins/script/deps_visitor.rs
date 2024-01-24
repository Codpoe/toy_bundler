use oxc::ast::{
  ast::{Expression, ImportDeclarationSpecifier, ImportOrExportKind, ModuleDeclaration},
  AstKind, Visit,
};

use crate::{module::ResolveKind, plugin::AnalyzeDep};

pub struct DepsVisitor {
  pub deps: Vec<AnalyzeDep>,
}

impl DepsVisitor {
  pub fn new() -> Self {
    Self { deps: vec![] }
  }
}

impl<'a> Visit<'a> for DepsVisitor {
  fn enter_node(&mut self, kind: AstKind<'a>) {
    match kind {
      AstKind::ModuleDeclaration(module_decl) => {
        if is_import_type(module_decl) {
          return;
        }

        match module_decl {
          ModuleDeclaration::ImportDeclaration(import_decl) => {
            // import a from 'a'
            self.deps.push(AnalyzeDep {
              source: import_decl.source.value.to_string(),
              resolve_kind: ResolveKind::Import,
            });
          }
          ModuleDeclaration::ExportNamedDeclaration(export_named_decl) => {
            // export { a } from 'a'
            if let Some(source) = &export_named_decl.source {
              self.deps.push(AnalyzeDep {
                source: source.value.to_string(),
                resolve_kind: ResolveKind::Import,
              });
            }
          }
          ModuleDeclaration::ExportAllDeclaration(export_all_decl) => {
            // export * from 'a'
            self.deps.push(AnalyzeDep {
              source: export_all_decl.source.value.to_string(),
              resolve_kind: ResolveKind::Import,
            });
          }
          _ => {}
        }
      }
      AstKind::ImportExpression(import_expr) => match &import_expr.source {
        Expression::StringLiteral(source) => {
          // import('a')
          self.deps.push(AnalyzeDep {
            source: source.value.to_string(),
            resolve_kind: ResolveKind::Import,
          });
        }
        _ => {}
      },
      _ => {}
    }
  }
}

/// 判断是否 import type。例如：
///
/// import type { A } from 'a'
///
/// import { type A } from 'a'
///
/// export type { A } from 'a'
///
/// export { type A } from 'a'
///
/// export type * from 'a'
fn is_import_type(decl: &ModuleDeclaration) -> bool {
  match decl {
    ModuleDeclaration::ImportDeclaration(decl) => {
      // import type { A } from 'a'
      if matches!(decl.import_kind, ImportOrExportKind::Type) {
        return true;
      }

      match &decl.specifiers {
        Some(specifiers) => specifiers.iter().all(|specifier| match specifier {
          ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
            // import { type A } from 'a'
            matches!(specifier.import_kind, ImportOrExportKind::Type)
          }
          _ => false,
        }),
        None => false,
      }
    }
    ModuleDeclaration::ExportNamedDeclaration(decl) => {
      if decl.source.is_none() {
        return false;
      }

      // export type { A } from 'a'
      if matches!(decl.export_kind, ImportOrExportKind::Type) {
        return true;
      }

      decl.specifiers.iter().all(|specifier| {
        // export { type A } from 'a'
        matches!(specifier.export_kind, ImportOrExportKind::Type)
      })
    }
    ModuleDeclaration::ExportAllDeclaration(decl) => {
      // export type * from 'a'
      matches!(decl.export_kind, ImportOrExportKind::Type)
    }
    _ => false,
  }
}
