use ouroboros::self_referencing;
use oxc::{allocator::Allocator, ast::ast::Program, parser::Parser, span::SourceType};
use std::{any::Any, collections::HashSet, ffi::OsStr, path::Path};
use swc_html::ast::Document;

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

  pub fn is_css(&self) -> bool {
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

#[derive(Debug)]
pub struct HtmlModuleMeta {
  pub ast: Document,
}

#[derive(Debug)]
pub struct CssModuleMeta {
  pub ast: Box<dyn Any + Send + Sync>,
}

pub struct ScriptProgram<'a>(pub Program<'a>);

unsafe impl<'a> Send for ScriptProgram<'a> {}
unsafe impl<'a> Sync for ScriptProgram<'a> {}

pub struct ScriptAllocator(Allocator);

unsafe impl Send for ScriptAllocator {}
unsafe impl Sync for ScriptAllocator {}

#[self_referencing]
pub struct ScriptAst {
  allocator: ScriptAllocator,
  source_text: String,
  source_type: SourceType,
  #[borrows(allocator, source_text, source_type)]
  #[not_covariant]
  pub program: &'this ScriptProgram<'this>,
}

impl ScriptAst {
  pub fn build(source_text: String, source_type: SourceType) -> Self {
    ScriptAstBuilder {
      allocator: ScriptAllocator(Allocator::default()),
      source_text,
      source_type,
      program_builder: |allocator, source_text, source_type| {
        let ret = Parser::new(&allocator.0, source_text, source_type.to_owned()).parse();
        allocator.0.alloc(ScriptProgram(ret.program))
      },
    }
    .build()
  }
}

pub struct ScriptModuleMeta {
  pub ast: ScriptAst,
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
