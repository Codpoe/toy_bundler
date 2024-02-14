use std::sync::{Arc, RwLock};

use crate::{
  config::Config,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupMap},
  oxc::OxcAstBuilder,
  plugin::{plugin_container::PluginContainer, Plugin},
  resource::{resource::ResourceMap, resource_pot::ResourcePotMap},
};

pub struct CompilationContext {
  pub config: Config,
  pub plugin_container: PluginContainer,
  pub module_graph: RwLock<ModuleGraph>,
  pub module_group_map: RwLock<ModuleGroupMap>,
  pub resource_pot_map: RwLock<ResourcePotMap>,
  pub resource_map: RwLock<ResourceMap>,
  pub ast_builder: OxcAstBuilder,
}

impl CompilationContext {
  pub fn new(mut config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Self {
    let ast_builder = OxcAstBuilder::build();
    let plugin_container = PluginContainer::new(plugins);

    plugin_container.config(&mut config).unwrap();

    Self {
      config,
      ast_builder,
      plugin_container,
      module_graph: RwLock::new(ModuleGraph::new()),
      module_group_map: RwLock::new(ModuleGroupMap::new()),
      resource_pot_map: RwLock::new(ResourcePotMap::new()),
      resource_map: RwLock::new(ResourceMap::new()),
    }
  }
}
