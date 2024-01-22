use crate::plugin::Plugin;

pub struct PluginScript {}

impl PluginScript {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginScript {
  fn name(&self) -> &str {
    "ToyPluginScript"
  }
}
