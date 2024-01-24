use std::{collections::HashMap, env};

use oxc_resolver::ResolveOptions;

#[derive(Debug)]
pub struct OutputConfig {
  pub dir: String,
}

#[derive(Debug)]
pub struct Config {
  pub root: String,
  pub input: HashMap<String, String>,
  pub output: OutputConfig,
  pub resolve: ResolveOptions,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      root: env::current_dir().unwrap().to_string_lossy().to_string(),
      input: HashMap::from([("main".to_string(), "./index.html".to_string())]),
      output: OutputConfig {
        dir: "./dist".to_string(),
      },
      resolve: ResolveOptions {
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
        ..ResolveOptions::default()
      },
    }
  }
}
