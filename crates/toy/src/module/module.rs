use std::{any::Any, collections::HashSet, ffi::OsStr, path::Path};
use swc_html::ast::Document;

#[derive(Debug, Clone)]
pub enum ModuleKind {
  Html,
  Css,
  Js,
  Jsx,
  Ts,
  Tsx,
  Asset,
  Custom(String),
}

impl ModuleKind {
  /// # Examples
  /// `"html"` -> `ModuleKind::Html`
  ///
  /// `"css"` -> `ModuleKind::Css`
  pub fn from_ext(ext: &str) -> Self {
    match ext {
      "html" => Self::Html,
      "css" => Self::Css,
      "js" => Self::Js,
      "jsx" => Self::Jsx,
      "ts" => Self::Ts,
      "tsx" => Self::Tsx,
      _ => Self::Custom(ext.to_string()),
    }
  }

  /// # Examples
  /// `"/a/b/c.html"` -> `ModuleKind::Html`
  ///
  /// `"/a/b/c.css"` -> `ModuleKind::Css`
  pub fn from_file_path<P: AsRef<OsStr>>(path: &P) -> Self {
    let ext = Path::new(path).extension().unwrap_or_default();
    Self::from_ext(ext.to_str().unwrap_or("unknown"))
  }
}

#[derive(Debug)]
pub enum ModuleMeta {
  Html(HtmlModuleMeta),
  Css(CssModuleMeta),
  Script(ScriptModuleMeta),
  Custom(Box<dyn Any + Send + Sync>),
}

#[derive(Debug)]
pub struct HtmlModuleMeta {
  pub ast: Document,
}

#[derive(Debug)]
pub struct CssModuleMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
pub struct ScriptModuleMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
pub struct Module {
  pub id: String,
  pub kind: ModuleKind,
  pub meta: ModuleMeta,
  pub module_groups: HashSet<String>,
}

impl Module {
  pub fn new(id: String, kind: ModuleKind) -> Self {
    Self {
      id,
      kind,
      meta: ModuleMeta::Custom(Box::new(())),
      module_groups: HashSet::new(),
    }
  }
}
