use oxc::ast::{
  ast::{Argument, CallExpression, Expression},
  AstBuilder, VisitMut,
};

/// 给入口 resource 注入模块系统 runtime
pub struct RuntimeVisitor<'a> {
  ast_builder: &'a AstBuilder<'a>,
  modules_object_expr: Expression<'a>,
  entry_id_expr: Expression<'a>,
}

impl<'a> RuntimeVisitor<'a> {
  pub fn new(
    ast_builder: &'a AstBuilder<'a>,
    modules_object_expr: Expression<'a>,
    entry_id_expr: Expression<'a>,
  ) -> Self {
    Self {
      ast_builder,
      modules_object_expr,
      entry_id_expr,
    }
  }
}

impl<'a> VisitMut<'a> for RuntimeVisitor<'a> {
  fn visit_call_expression(&mut self, expr: &mut CallExpression<'a>) {
    if let Expression::ParenthesizedExpression(parenthesized_expr) = &expr.callee {
      if let Expression::FunctionExpression(fn_expr) = &parenthesized_expr.expression {
        if let Some(id) = &fn_expr.id {
          if id.name.as_str() == "bootstrap" {
            expr.arguments.clear();
            expr.arguments.extend(vec![
              Argument::Expression(self.ast_builder.copy(&self.modules_object_expr)),
              Argument::Expression(self.ast_builder.copy(&self.entry_id_expr)),
            ]);
          }
        }
      }
    }
  }
}
