use std::convert::Infallible;

use lightningcss::{
  rules::CssRule,
  values::url::Url,
  visit_types,
  visitor::{VisitTypes, Visitor},
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

impl<'a> Visitor<'a> for DepsVisitor {
  type Error = Infallible;

  fn visit_types(&self) -> VisitTypes {
    visit_types!(RULES | URLS)
  }

  fn visit_rule(&mut self, rule: &mut CssRule<'a>) -> Result<(), Self::Error> {
    if let CssRule::Import(import_rule) = rule {
      self.deps.push(AnalyzeDep {
        source: import_rule.url.to_string(),
        resolve_kind: ResolveKind::CssAtImport,
      });
    }

    Ok(())
  }

  fn visit_url(&mut self, _url: &mut Url<'a>) -> Result<(), Self::Error> {
    // TODO: url deps. eg.
    // - background: url(...)
    Ok(())
  }
}
