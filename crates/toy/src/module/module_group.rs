use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ModuleGroup {
  /// the entry of the module group
  pub id: String,
  /// the modules that this group has
  module_ids: Vec<String>,
  /// the resource pots this group merged to
  resource_pot_ids: Vec<String>,
}

impl ModuleGroup {
  pub fn new(id: String) -> Self {
    Self {
      id: id.clone(),
      module_ids: vec![id],
      resource_pot_ids: vec![],
    }
  }

  pub fn add_module_id(&mut self, module_id: String) {
    self.module_ids.push(module_id);
  }

  pub fn module_ids(&self) -> &Vec<String> {
    &self.module_ids
  }

  pub fn add_resource_pot_id(&mut self, resource_pot_id: String) {
    self.resource_pot_ids.push(resource_pot_id);
  }

  pub fn resource_pot_ids(&self) -> &Vec<String> {
    &self.resource_pot_ids
  }
}

pub type ModuleGroupMap = HashMap<String, ModuleGroup>;
