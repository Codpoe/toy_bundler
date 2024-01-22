use crate::plugin::Plugin;

pub struct PluginRuntime {}

impl PluginRuntime {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginRuntime {
  fn name(&self) -> &str {
    "ToyPluginRuntime"
  }
}
