use std::{
  any::Any,
  collections::{HashMap, HashSet},
};

use swc_html::ast::Document;

use crate::module::module::ModuleKind;

#[derive(Debug)]
pub enum ResourcePotKind {
  Runtime,
  Html,
  Css,
  Js,
  Asset,
  Custom(String),
}

impl ResourcePotKind {
  pub fn from_module_kind(module_kind: ModuleKind) -> Self {
    match module_kind {
      ModuleKind::Html => Self::Html,
      ModuleKind::Css => Self::Css,
      ModuleKind::Js | ModuleKind::Jsx | ModuleKind::Ts | ModuleKind::Tsx => Self::Js,
      ModuleKind::Asset => Self::Asset,
      ModuleKind::Custom(c) => Self::Custom(c.clone()),
    }
  }
}

#[derive(Debug)]
pub enum ResourcePotMeta {
  Html(HtmlResourcePotMeta),
  Css(CssResourcePotMeta),
  Js(JsResourcePotMeta),
  Custom(Box<dyn Any + Send + Sync>),
}

#[derive(Debug)]
pub struct HtmlResourcePotMeta {
  pub ast: Document,
}

#[derive(Debug)]
pub struct CssResourcePotMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
pub struct JsResourcePotMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
pub struct ResourcePot {
  pub id: String,
  pub kind: ResourcePotKind,
  pub module_group_id: String,
  pub module_ids: HashSet<String>,
  pub resource_ids: HashSet<String>,
  pub meta: ResourcePotMeta,
}

impl ResourcePot {
  pub fn new(id: String, kind: ResourcePotKind, module_group_id: String) -> Self {
    Self {
      id,
      kind,
      module_group_id,
      module_ids: HashSet::new(),
      resource_ids: HashSet::new(),
      meta: ResourcePotMeta::Custom(Box::new(())),
    }
  }
}

pub type ResourcePotMap = HashMap<String, ResourcePot>;
