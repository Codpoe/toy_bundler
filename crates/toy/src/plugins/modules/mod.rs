use std::{
  collections::{HashMap, HashSet, VecDeque},
  sync::Arc,
};

use crate::{
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    module_group::{ModuleGroup, ModuleGroupMap},
    ResolveKind,
  },
  plugin::Plugin,
  resource::resource_pot::{ResourcePot, ResourcePotKind, ResourcePotMap},
};

pub struct PluginModules {}

impl PluginModules {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for PluginModules {
  fn name(&self) -> &'static str {
    "ToyPluginModules"
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ModuleGroupMap>> {
    let mut module_group_map = ModuleGroupMap::new();

    // 从入口开始遍历模块，分析依赖，把静态依赖分组为同一个 module_group
    // (动态依赖也会被当作入口去遍历分析)
    for entry_id in module_graph.entries.clone() {
      let (module_group, dynamic_deps) = module_group_from_entry(entry_id, module_graph)?;
      module_group_map.insert(module_group.id.clone(), module_group);

      let mut queue = VecDeque::from(dynamic_deps);
      let mut seen = HashSet::new();

      while queue.len() > 0 {
        let head = queue.pop_front().unwrap();

        if seen.contains(&head) {
          continue;
        }

        seen.insert(head.clone());

        let (module_group, dynamic_deps) = module_group_from_entry(head, module_graph)?;

        module_group_map.insert(module_group.id.clone(), module_group);
        queue.extend(dynamic_deps);
      }
    }

    Ok(Some(module_group_map))
  }

  fn merge_modules(
    &self,
    module_group_map: &mut ModuleGroupMap,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourcePotMap>> {
    // 把 module_group 里的模块按类型进行合并。
    // 一个 module_group 里可能既有 JS 模块，也有 CSS 模块，
    // 需要把它们分别合并成一个 JS resource_pot 和一个 CSS resource_pot。
    let mut module_graph = context.module_graph.write().unwrap();
    let mut resource_pot_map = HashMap::new();

    for module_group in module_group_map.values_mut() {
      let mut resource_pot_by_kind = HashMap::<ResourcePotKind, ResourcePot>::new();

      for module_id in module_group.module_ids().clone() {
        let module = module_graph.module_mut(&module_id).unwrap();
        let resource_pot_kind = ResourcePotKind::from_module_kind(module.kind.clone());

        if resource_pot_by_kind.contains_key(&resource_pot_kind) {
          // 已存在该类型的 resource_pot
          let resource_pot = resource_pot_by_kind.get_mut(&resource_pot_kind).unwrap();

          resource_pot.module_ids.push(module_id.clone());
        } else {
          // 不存在该类型的 resource_pot，
          // 则根据这个 module_id 创建相应的 resource_pot
          let resource_pot = ResourcePot::new(
            module_id.clone(),
            resource_pot_kind.clone(),
            module_group.id.clone(),
          );

          module_group.add_resource_pot_id(resource_pot.id.clone());
          resource_pot_by_kind.insert(resource_pot_kind.clone(), resource_pot);
        }
      }

      for resource_pot in resource_pot_by_kind.into_values() {
        resource_pot_map.insert(resource_pot.id.clone(), resource_pot);
      }
    }

    Ok(Some(resource_pot_map))
  }
}

fn module_group_from_entry(
  id: String,
  module_graph: &mut ModuleGraph,
) -> Result<(ModuleGroup, Vec<String>)> {
  let mut module_group = ModuleGroup::new(id.clone());

  module_graph
    .module_mut(&id)
    .unwrap()
    .module_groups
    .insert(module_group.id.clone());

  let mut seen = HashSet::new();
  let mut static_deps: Vec<String> = vec![];
  let mut dynamic_deps: Vec<String> = vec![];

  collect_module_deps(
    module_graph,
    &id,
    &mut static_deps,
    &mut dynamic_deps,
    &mut seen,
  )?;

  for dep_id in static_deps {
    module_group.add_module_id(dep_id.clone());
    module_graph
      .module_mut(&dep_id)
      .unwrap()
      .module_groups
      .insert(module_group.id.clone());
  }

  // for (dep_id, edge) in module_graph.dependencies(&id)? {
  //   if matches!(edge.kind, ResolveKind::DynamicImport) {
  //     dynamic_deps.push(dep_id);
  //   } else {
  //     let mut queue = VecDeque::from([dep_id.clone()]);

  //     while queue.len() > 0 {
  //       let head = queue.pop_front().unwrap();

  //       if seen.contains(&head) {
  //         continue;
  //       }

  //       seen.insert(head.clone());

  //       module_group.add_module_id(head.clone());
  //       module_graph
  //         .module_mut(&dep_id)
  //         .unwrap()
  //         .module_groups
  //         .insert(module_group.id.clone());

  //       println!("@@@ deps {:#?}", module_graph.dependencies(&head)?);

  //       for (dep_id, edge) in module_graph.dependencies(&head)? {
  //         if matches!(edge.kind, ResolveKind::DynamicImport) {
  //           dynamic_deps.push(dep_id);
  //         } else {
  //           queue.push_back(dep_id);
  //         }
  //       }
  //     }
  //   }
  // }

  Ok((module_group, dynamic_deps))
}

fn collect_module_deps(
  module_graph: &mut ModuleGraph,
  module_id: &str,
  static_deps: &mut Vec<String>,
  dynamic_deps: &mut Vec<String>,
  seen: &mut HashSet<String>,
) -> Result<()> {
  if seen.contains(module_id) {
    return Ok(());
  }

  seen.insert(module_id.to_string());

  for (dep_id, edge) in module_graph.dependencies(module_id)? {
    if matches!(edge.kind, ResolveKind::DynamicImport) {
      dynamic_deps.push(dep_id);
    } else {
      static_deps.push(dep_id.clone());
      collect_module_deps(module_graph, &dep_id, static_deps, dynamic_deps, seen)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_module_group_from_entry() {
    let mut module_graph = ModuleGraph::mock_module_graph();

    // entry "a"
    let (module_group, dynamic_deps) =
      module_group_from_entry("a".to_string(), &mut module_graph).unwrap();

    let mut right_module_group = ModuleGroup::new("a".to_string());

    right_module_group.add_module_id("c".to_string());
    right_module_group.add_module_id("f".to_string());

    assert_eq!(module_group, right_module_group);
    assert_eq!(dynamic_deps, vec!["d".to_string()]);

    // entry "b"
    let (module_group, dynamic_deps) =
      module_group_from_entry("b".to_string(), &mut module_graph).unwrap();

    let mut right_module_group = ModuleGroup::new("b".to_string());
    let right_dynamic_deps: Vec<String> = vec![];

    right_module_group.add_module_id("e".to_string());

    assert_eq!(module_group, right_module_group);
    assert_eq!(dynamic_deps, right_dynamic_deps);

    // dynamic entry "d"
    let (module_group, dynamic_deps) =
      module_group_from_entry("d".to_string(), &mut module_graph).unwrap();

    let mut right_module_group = ModuleGroup::new("d".to_string());
    let right_dynamic_deps: Vec<String> = vec![];

    right_module_group.add_module_id("f".to_string());

    assert_eq!(module_group, right_module_group);
    assert_eq!(dynamic_deps, right_dynamic_deps);
  }
}
