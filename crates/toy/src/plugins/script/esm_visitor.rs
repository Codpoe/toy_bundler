use oxc::{
  allocator::Allocator,
  ast::{
    ast::{BindingIdentifier, FormalParameterKind, FunctionType, Modifiers},
    AstBuilder, AstKind, VisitMut,
  },
  span::Span,
};

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
  allocator: &'a Allocator,
}

impl<'a> EsmVisitor<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self { allocator }
  }
}

impl<'a> VisitMut<'a> for EsmVisitor<'a> {
  fn enter_node(&mut self, kind: AstKind<'a>) {}
}
