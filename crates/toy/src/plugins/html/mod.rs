use crate::plugin::Plugin;

pub struct PluginHtml {}

impl PluginHtml {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginHtml {
  fn name(&self) -> &str {
    "ToyPluginHtml"
  }
}
