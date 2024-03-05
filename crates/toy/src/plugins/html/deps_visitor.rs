use swc_common::util::take::Take;
use swc_html::{
  ast::{Document, Element},
  visit::{Visit, VisitWith},
};

use crate::{module::ResolveKind, plugin::AnalyzeDep};

pub struct DepsVisitor {
  pub deps: Vec<AnalyzeDep>,
}

impl DepsVisitor {
  pub fn new() -> Self {
    Self { deps: vec![] }
  }

  pub fn analyze_deps(&mut self, document: &Document) -> Vec<AnalyzeDep> {
    document.visit_with(self);
    self.deps.take()
  }
}

impl Visit for DepsVisitor {
  fn visit_element(&mut self, el: &Element) {
    match el.tag_name.as_str() {
      "script" => {
        if let Some(src) = get_script_src(el) {
          self.deps.push(AnalyzeDep {
            source: src,
            resolve_kind: ResolveKind::ScriptSrc,
          });
        }
      }
      "link" => {
        if let Some(href) = get_link_href(el) {
          self.deps.push(AnalyzeDep {
            source: href,
            resolve_kind: ResolveKind::LinkHref,
          });
        }
      }
      _ => {}
    }

    el.visit_children_with(self);
  }
}

pub fn get_script_src(el: &Element) -> Option<String> {
  if el.tag_name.to_string() == "script" {
    let src_attr = el
      .attributes
      .iter()
      .find(|attr| attr.name.to_string() == "src");

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        return Some(value.to_string());
      }
    }
  }

  None
}

pub fn get_link_href(el: &Element) -> Option<String> {
  if el.tag_name.to_string() == "link" {
    let href_attr = el
      .attributes
      .iter()
      .find(|attr| attr.name.to_string() == "href");

    if let Some(href_attr) = href_attr {
      if let Some(value) = &href_attr.value {
        return Some(value.to_string());
      }
    }
  }

  None
}
