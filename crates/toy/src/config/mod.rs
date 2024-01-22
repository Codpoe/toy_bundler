use std::collections::HashMap;

#[derive(Debug)]
pub struct OutputConfig {
  dir: String,
}

#[derive(Debug)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub output: OutputConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      input: HashMap::from([("main".to_string(), "index.html".to_string())]),
      output: OutputConfig {
        dir: "dist".to_string(),
      },
    }
  }
}
