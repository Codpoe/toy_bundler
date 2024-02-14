use std::{any::Any, collections::HashSet, ffi::OsStr, path::Path};
use swc_html::ast::Document;

use crate::oxc::{OxcProgram, OxcProgramWrapper};

#[derive(Debug, Clone, PartialEq, Eq)]
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
      "js" | "cjs" | "mjs" => Self::Js,
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

  pub fn is_html(&self) -> bool {
    matches!(self, Self::Html)
  }

  pub fn is_style(&self) -> bool {
    matches!(self, Self::Css)
  }

  pub fn is_script(&self) -> bool {
    matches!(self, Self::Js | Self::Jsx | Self::Ts | Self::Tsx)
  }
}

pub enum ModuleMeta {
  Html(HtmlModuleMeta),
  Css(CssModuleMeta),
  Script(ScriptModuleMeta),
  Custom(Box<dyn Any + Send + Sync>),
}

impl ModuleMeta {
  pub fn as_html(&self) -> &HtmlModuleMeta {
    match self {
      Self::Html(meta) => meta,
      _ => unreachable!("ModuleMeta `as_html()` failed"),
    }
  }

  pub fn as_css(&self) -> &CssModuleMeta {
    match self {
      Self::Css(meta) => meta,
      _ => unreachable!("ModuleMeta `as_css()` failed"),
    }
  }

  pub fn as_script(&self) -> &ScriptModuleMeta {
    match self {
      Self::Script(meta) => meta,
      _ => unreachable!("ModuleMeta `as_script()` failed"),
    }
  }
}

#[derive(Debug)]
pub struct HtmlModuleMeta {
  pub ast: Document,
}

#[derive(Debug)]
pub struct CssModuleMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

pub struct ScriptModuleMeta {
  pub code: String,
  pub ast: OxcProgram,
}

pub struct Module {
  pub id: String,
  pub kind: ModuleKind,
  pub meta: ModuleMeta,
  pub module_groups: HashSet<String>,
}

impl Module {
  pub fn new(id: String, kind: ModuleKind, meta: Option<ModuleMeta>) -> Self {
    Self {
      id,
      kind,
      meta: meta.unwrap_or(ModuleMeta::Custom(Box::new(()))),
      module_groups: HashSet::new(),
    }
  }
}
