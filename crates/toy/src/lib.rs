use std::sync::Arc;

use config::Config;
use context::CompilationContext;
use error::Result;
use plugin::Plugin;
use plugins::{
  modules::PluginModules, resolve::PluginResolve, resources::PluginResources, script::PluginScript,
};

mod build;
mod config;
mod context;
pub mod error;
mod generate;
mod module;
mod oxc;
mod plugin;
mod plugins;
mod resource;

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

pub struct Compiler {
  context: Arc<CompilationContext>,
}

impl Compiler {
  pub fn new(config: Config, mut plugins: Vec<Arc<dyn Plugin>>) -> Self {
    let mut final_plugins: Vec<Arc<dyn Plugin>> = vec![
      Arc::new(PluginResolve::new(config.resolve.clone())),
      Arc::new(PluginScript::new()),
      Arc::new(PluginModules::new()),
      Arc::new(PluginResources::new()),
    ];

    final_plugins.append(&mut plugins);
    final_plugins.sort_by_key(|plugin| plugin.priority());

    Self {
      context: Arc::new(CompilationContext::new(config, final_plugins)),
    }
  }

  pub fn compile(&mut self) -> Result<()> {
    self.build()?;

    self.generate()?;

    println!(">>> done");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, fs};

  use super::*;

  #[test]
  fn it_works() {
    let mut compiler = Compiler::new(
      Config {
        root: fs::canonicalize("../../fixtures/basic")
          .unwrap()
          .to_string_lossy()
          .to_string(),
        input: HashMap::from([("main".to_string(), "./index.js".to_string())]),
        ..Config::default()
      },
      vec![],
    );
    compiler.compile().unwrap();
  }
}
