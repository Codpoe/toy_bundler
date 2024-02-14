use std::{fs, path::PathBuf, sync::Arc};

use crate::{
  context::CompilationContext, error::Result, plugin::Plugin, resource::resource::ResourceMap,
};

pub struct PluginResources {}

impl PluginResources {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginResources {
  fn name(&self) -> &str {
    "ToyPluginResources"
  }

  fn write_resources(
    &self,
    resources: &mut ResourceMap,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    let root = PathBuf::from(&context.config.root);
    let out_dir = root.join(&context.config.output.dir);

    fs::create_dir_all(out_dir.clone()).unwrap();
    fs::remove_dir_all(out_dir.clone()).unwrap();

    for resource in resources.values_mut() {
      if !resource.emitted {
        let out_path = out_dir.join(&resource.name);

        fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        fs::write(out_path, &resource.content).unwrap();
        resource.emitted = true;
      }
    }

    Ok(())
  }
}
