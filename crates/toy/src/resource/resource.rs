use std::collections::HashMap;

#[derive(Debug)]
pub enum ResourceKind {
  Runtime,
  Html,
  Css,
  Js,
  SourceMap,
  Asset,
  Custom(String),
}

#[derive(Debug)]
pub struct Resource {
  pub name: String,
  pub content: String,
  pub resource_kind: ResourceKind,
  /// whether the resource is emitted
  pub emitted: bool,
  pub resource_pot_id: String,
}

pub type ResourceMap = HashMap<String, Resource>;
