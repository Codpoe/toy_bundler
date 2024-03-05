use oxc::{allocator::Box as OxcBox, ast::ast::Program};
use std::{any::Any, collections::HashMap};
use swc_html::ast::Document;

use crate::{
  module::module::{ModuleKind, ScriptModuleMeta},
  oxc::{OxcProgram, OxcProgramWrapper},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl ResourcePotMeta {
  pub fn as_html(&self) -> &HtmlResourcePotMeta {
    match self {
      Self::Html(meta) => meta,
      _ => unreachable!("ResourcePotMeta `as_html()` failed"),
    }
  }

  pub fn as_html_mut(&mut self) -> &mut HtmlResourcePotMeta {
    match self {
      Self::Html(meta) => meta,
      _ => unreachable!("ResourcePotMeta `as_html()` failed"),
    }
  }

  pub fn as_css(&self) -> &CssResourcePotMeta {
    match self {
      Self::Css(meta) => meta,
      _ => unreachable!("ResourcePotMeta `as_css()` failed"),
    }
  }

  pub fn as_js(&self) -> &JsResourcePotMeta {
    match self {
      Self::Js(meta) => meta,
      _ => unreachable!("ResourcePotMeta `as_js()` failed"),
    }
  }
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
  pub ast: Option<Box<dyn Any + Send + Sync>>,
  pub code: String,
}

#[derive(Debug)]
pub struct ResourcePot {
  pub id: String,
  pub kind: ResourcePotKind,
  pub module_group_id: String,
  pub module_ids: Vec<String>,
  pub resource_ids: Vec<String>,
  pub meta: ResourcePotMeta,
}

impl ResourcePot {
  pub fn new(id: String, kind: ResourcePotKind, module_group_id: String) -> Self {
    Self {
      id: id.clone(),
      kind,
      module_group_id,
      module_ids: vec![id],
      resource_ids: vec![],
      meta: ResourcePotMeta::Custom(Box::new(())),
    }
  }
}

pub type ResourcePotMap = HashMap<String, ResourcePot>;
