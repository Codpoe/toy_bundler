use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use oxc_resolver::{ResolveOptions, Resolver};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{Plugin, ResolveHookParams, ResolveHookResult},
  utils::fulfill_root_prefix,
};

pub struct PluginResolve {
  resolver: Resolver,
}

impl PluginResolve {
  pub fn new(options: ResolveOptions) -> Self {
    Self {
      resolver: Resolver::new(options),
    }
  }
}

impl Plugin for PluginResolve {
  fn name(&self) -> &str {
    "ToyPluginResolve"
  }

  fn resolve(
    &self,
    params: &ResolveHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<ResolveHookResult>> {
    let root = &context.config.root;

    let base = params
      .importer
      .as_ref()
      .map(|importer| {
        PathBuf::from(fulfill_root_prefix(importer, root))
          .parent()
          .unwrap()
          .to_string_lossy()
          .to_string()
      })
      .unwrap_or(context.config.root.clone());

    resolve_id(&self.resolver, &params.source, &base, &context.config.root).map(|v| Some(v))
  }
}

fn resolve_id(
  resolver: &Resolver,
  source: &str,
  base: &str,
  root: &str,
) -> Result<ResolveHookResult> {
  // extract query
  let (source, query_str) = source.split_once('?').unwrap_or((source, ""));
  let query = parse_query(query_str);

  let resolution =
    resolver
      .resolve(base.to_string(), source)
      .map_err(|err| CompilationError::ResolveError {
        src: source.to_string(),
        base: base.to_string(),
        source: Some(Box::new(err)),
      })?;

  let mut id = resolution.path().to_string_lossy().to_string();
  id = "root:".to_string() + to_relative(&id, root).as_str();

  Ok(ResolveHookResult {
    id,
    query,
    external: false,
  })
}

fn parse_query(query_str: &str) -> HashMap<String, String> {
  let mut query: HashMap<String, String> = HashMap::new();

  if !query_str.is_empty() {
    let parts: Vec<&str> = query_str.split('&').collect();

    for part in parts {
      let (key, value) = part.split_once('=').unwrap_or((part, ""));
      query.insert(key.to_string(), value.to_string());
    }
  }

  query
}

/// Convert a path string into a relative path, if it's inside the root.
/// If it's outside of the root or it's not an absolute path, return it as is.
fn to_relative(path: &str, root: &str) -> String {
  let path = Path::new(path);
  let root = Path::new(root);

  // If the path is not a child of the root, or not absolute, return it as is.
  if !path.is_absolute() || path == root || !path.starts_with(root) {
    return path.to_string_lossy().to_string();
  }

  // Otherwise, it's a child of the root. Convert it into a relative path of the root.
  let relative = path
    .strip_prefix(root)
    .unwrap()
    .to_string_lossy()
    .to_string();

  relative
}

#[cfg(test)]
mod tests {
  use std::{env, fs};

  use super::*;

  #[test]
  fn test_parse_query() {
    assert!(parse_query("").is_empty());

    let query = parse_query("import&foo=bar&bar=baz");
    assert_eq!(query.get("import").unwrap(), "");
    assert_eq!(query.get("foo").unwrap(), "bar");
    assert_eq!(query.get("bar").unwrap(), "baz");
  }

  #[test]
  fn test_resolve_id() {
    let resolver = Resolver::new(ResolveOptions {
      extensions: vec![
        ".js".to_string(),
        ".jsx".to_string(),
        ".ts".to_string(),
        ".tsx".to_string(),
      ],
      main_fields: vec![
        "browser".to_string(),
        "module".to_string(),
        "main".to_string(),
      ],
      main_files: vec!["index".to_string()],
      ..ResolveOptions::default()
    });

    let root = fs::canonicalize("../../fixtures/basic")
      .unwrap()
      .to_string_lossy()
      .to_string();

    let res = resolve_id(
      &resolver,
      "../../fixtures/basic/index?foo=bar",
      env::current_dir().unwrap().to_str().unwrap(),
      &root,
    )
    .unwrap();

    assert_eq!(res.id, "root:index.js");

    assert_eq!(res.query.get("foo").unwrap(), "bar");
    assert!(!res.external);

    let source = fs::canonicalize("../../fixtures/basic/index.js").unwrap();

    let res = resolve_id(
      &resolver,
      source.parent().unwrap().to_str().unwrap(),
      env::current_dir().unwrap().to_str().unwrap(),
      &root,
    )
    .unwrap();

    assert_eq!(res.id, "root:index.js");

    assert!(res.query.is_empty());
    assert!(!res.external);
  }
}
