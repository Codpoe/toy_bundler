use swc_common::DUMMY_SP;
use swc_html::{
  ast::{Attribute, Child, Document, Element, Namespace, Text},
  visit::{VisitMut, VisitMutWith},
};

use super::deps_visitor::{get_link_href, get_script_src};

pub struct ResourcesInjector {
  /// 原来 html 的依赖 url
  deps: Vec<String>,
  /// 需要注入的 css 资源
  css_resources: Vec<String>,
  /// 需要注入的 js 资源
  js_resources: Vec<String>,
}

impl ResourcesInjector {
  pub fn new(deps: Vec<String>, css_resources: Vec<String>, js_resources: Vec<String>) -> Self {
    ResourcesInjector {
      deps,
      css_resources,
      js_resources,
    }
  }

  pub fn inject(&mut self, document: &mut Document) {
    document.visit_mut_with(self);
  }
}

impl VisitMut for ResourcesInjector {
  fn visit_mut_children(&mut self, children: &mut std::vec::Vec<Child>) {
    let mut children_to_remove = vec![];
    // remove all existing <href /> and <script /> deps
    for (i, child) in children.iter().enumerate() {
      if let Child::Element(el) = child {
        if let Some(src_or_href) = get_script_src(el).or_else(|| get_link_href(el)) {
          if self.deps.contains(&src_or_href) {
            children_to_remove.push(i);
          }
        }
      }
    }

    children_to_remove.into_iter().for_each(|i| {
      children.remove(i);
    });
  }

  fn visit_mut_element(&mut self, el: &mut Element) {
    if el.tag_name.to_string() == "head" {
      // 注入 css 资源
      for css in &self.css_resources {
        el.children.push(Child::Element(create_element(
          "link",
          Some(vec![("rel", "stylesheet"), ("href", css)]),
          None,
        )));
      }
    } else if el.tag_name.to_string() == "body" {
      // 注入 js 资源
      for js in &self.js_resources {
        el.children.push(Child::Element(create_element(
          "script",
          Some(vec![("src", js)]),
          None,
        )));
      }
    }
  }
}

fn create_element(name: &str, attrs: Option<Vec<(&str, &str)>>, text: Option<&str>) -> Element {
  Element {
    span: DUMMY_SP,
    tag_name: name.into(),
    namespace: Namespace::HTML,
    attributes: attrs.map_or(vec![], |attrs| {
      attrs
        .into_iter()
        .map(|(name, value)| Attribute {
          span: DUMMY_SP,
          namespace: None,
          prefix: None,
          name: name.into(),
          raw_name: None,
          value: Some(value.into()),
          raw_value: None,
        })
        .collect()
    }),
    children: text.map_or(vec![], |text| {
      vec![Child::Text(Text {
        span: DUMMY_SP,
        data: text.into(),
        raw: None,
      })]
    }),
    content: None,
    is_self_closing: false,
  }
}
