use crate::plugin::Plugin;

pub struct PluginCss {}

impl PluginCss {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginCss {
  fn name(&self) -> &str {
    "ToyPluginCss"
  }
}
