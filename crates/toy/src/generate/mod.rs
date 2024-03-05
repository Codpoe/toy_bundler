use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{error::Result, Compiler};

impl Compiler {
  pub(crate) fn generate(&mut self) -> Result<()> {
    // generate_start
    self
      .context
      .plugin_container
      .generate_start(&self.context)?;

    println!(">>> generate_start");

    // analyze_module_graph -> build module_group_map
    let mut module_graph = self.context.module_graph.write().unwrap();

    let ret_module_group_map = self
      .context
      .plugin_container
      .analyze_module_graph(&mut module_graph, &self.context)?
      .unwrap();

    let mut module_group_map = self.context.module_group_map.write().unwrap();
    *module_group_map = ret_module_group_map;
    println!(">>> module_group_map {:#?}", module_group_map);

    drop(module_graph);

    // merge_modules -> build resource_pot_map
    let ret_resource_pot_map = self
      .context
      .plugin_container
      .merge_modules(&mut module_group_map, &self.context)?
      .unwrap();

    drop(module_group_map);

    let mut resource_pot_map = self.context.resource_pot_map.write().unwrap();
    *resource_pot_map = ret_resource_pot_map;

    println!(">>> resource_pot_map {:#?}", resource_pot_map);

    // 1. render_resource_pot
    // 2. generate_resources
    resource_pot_map
      .values_mut()
      .collect::<Vec<_>>()
      .into_par_iter()
      .try_for_each(|mut resource_pot| {
        // render_resource_pot
        self
          .context
          .plugin_container
          .render_resource_pot(&mut resource_pot, &self.context)?;

        println!(">>> render_resource_pot {:#?}", resource_pot);

        // generate_resources
        let resources = self
          .context
          .plugin_container
          .generate_resources(&mut resource_pot, &self.context)?;

        println!(">>> generate_resources {:#?}", resources);

        if let Some(resources) = resources {
          let mut resource_map = self.context.resource_map.write().unwrap();
          resource_map.extend(resources);
          drop(resource_map);
        }

        Ok(())
      })?;

    drop(resource_pot_map);

    let mut resource_map = self.context.resource_map.write().unwrap();

    // write_resources -> could be emitted to filesystem
    self
      .context
      .plugin_container
      .write_resources(&mut resource_map, &self.context)?;

    println!(">>> write_resources");

    drop(resource_map);

    // generate_end
    self.context.plugin_container.generate_end(&self.context)?;

    println!(">>> generate_end");

    Ok(())
  }
}
