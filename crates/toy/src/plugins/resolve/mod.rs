use std::{collections::HashMap, sync::Arc};

use oxc_resolver::{ResolveOptions, Resolver};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{Plugin, ResolveHookParams, ResolveHookResult},
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
    resolve_id(
      &self.resolver,
      &params.source,
      &params
        .importer
        .clone()
        .unwrap_or(context.config.root.clone()),
    )
    .map(|v| Some(v))
  }
}

fn resolve_id(resolver: &Resolver, source: &str, base: &str) -> Result<ResolveHookResult> {
  // extract query
  let (source, query_str) = source.split_once('?').unwrap_or((source, ""));
  let query = parse_query(query_str);

  let resolution =
    resolver
      .resolve(base.to_string(), source)
      .map_err(|err| CompilationError::ResolveError {
        source: Some(Box::new(err)),
      })?;

  Ok(ResolveHookResult {
    id: resolution.path().to_string_lossy().to_string(),
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
        ".rs".to_string(),
      ],
      main_fields: vec![
        "browser".to_string(),
        "module".to_string(),
        "main".to_string(),
      ],
      main_files: vec!["index".to_string(), "lib".to_string()],
      ..ResolveOptions::default()
    });

    let res = resolve_id(
      &resolver,
      "./src/lib?foo=bar",
      env::current_dir().unwrap().to_str().unwrap(),
    )
    .unwrap();

    assert_eq!(
      res.id,
      fs::canonicalize("./src/lib.rs")
        .unwrap()
        .to_string_lossy()
        .to_string()
    );

    assert_eq!(res.query.get("foo").unwrap(), "bar");
    assert!(!res.external);

    let source = format!(
      "{}/{}",
      env::current_dir().unwrap().to_string_lossy().to_string(),
      "src"
    );

    let res = resolve_id(
      &resolver,
      &source,
      env::current_dir().unwrap().to_str().unwrap(),
    )
    .unwrap();

    assert_eq!(
      res.id,
      fs::canonicalize("./src/lib.rs")
        .unwrap()
        .to_string_lossy()
        .to_string()
    );

    assert!(res.query.is_empty());
    assert!(!res.external);
  }
}
