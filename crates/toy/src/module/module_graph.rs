use std::collections::{HashMap, HashSet};

use petgraph::{
  stable_graph::{NodeIndex, StableDiGraph},
  Direction,
};

use crate::error::{CompilationError, Result};

use super::{module::Module, ResolveKind};

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleGraphEdge {
  pub kind: ResolveKind,
  /// the source of this edge, for example, `./index.css`
  pub source: String,
  /// the order of this edge, for example, for:
  /// ```js
  /// import a from './a';
  /// import b from './b';
  /// ```
  /// the edge `./a`'s order is 0 and `./b`'s order is 1 (starting from 0).
  pub order: usize,
}

pub struct ModuleGraph {
  graph: StableDiGraph<Module, ModuleGraphEdge>,
  id_to_index: HashMap<String, NodeIndex>,
  pub entries: HashSet<String>,
  pub entries_in_html: HashSet<String>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      graph: StableDiGraph::new(),
      id_to_index: HashMap::new(),
      entries: HashSet::new(),
      entries_in_html: HashSet::new(),
    }
  }

  pub fn add_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.graph.add_node(module);
    self.id_to_index.insert(id, index);
  }

  pub fn module(&self, id: &str) -> Option<&Module> {
    let index = self.id_to_index.get(id);

    if let Some(index) = index {
      self.graph.node_weight(*index)
    } else {
      None
    }
  }

  pub fn module_mut(&mut self, id: &str) -> Option<&mut Module> {
    let index = self.id_to_index.get(id);

    if let Some(index) = index {
      self.graph.node_weight_mut(*index)
    } else {
      None
    }
  }

  pub fn add_edge(&mut self, from: &str, to: &str, edge_info: ModuleGraphEdge) -> Result<()> {
    let from_index = self.id_to_index.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        "from node \"{}\" is not found in module graph",
        from
      ))
    })?;

    let to_index = self.id_to_index.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        "from node \"{}\" is not found in module graph",
        from
      ))
    })?;

    self.graph.add_edge(*from_index, *to_index, edge_info);

    Ok(())
  }

  /// 获取模块的依赖项
  pub fn dependencies(&self, id: &str) -> Result<Vec<(String, ModuleGraphEdge)>> {
    let index = self.id_to_index.get(id).ok_or_else(|| {
      CompilationError::GenericError(format!("id \"{}\" is not found in module graph", id))
    })?;

    let mut edges = self
      .graph
      .neighbors_directed(*index, Direction::Outgoing)
      .detach();

    let mut deps = Vec::new();

    while let Some((edge_index, node_index)) = edges.next(&self.graph) {
      deps.push((
        self.graph[node_index].id.clone(),
        self.graph[edge_index].clone(),
      ));
    }

    deps.sort_by(|a, b| a.1.order.cmp(&b.1.order));

    Ok(deps)
  }

  pub fn is_entry_module(&self, id: &str, check_entries_in_html: bool) -> bool {
    let ret = self.entries.contains(id);

    if check_entries_in_html {
      ret || self.entries_in_html.contains(id)
    } else {
      ret
    }
  }

  /// mock a module graph:
  /// ```text
  ///   a  b
  ///  / \  \
  /// c  d   e
  ///  \ /
  ///   f
  /// ```
  /// - static import: `a -> c`, `c -> f`, `d -> f`, `b -> e`
  /// - dynamic import: `a -> d`
  #[cfg(test)]
  pub fn mock_module_graph() -> ModuleGraph {
    use super::module::ModuleKind;

    let mut module_graph = ModuleGraph::new();

    let module_a = Module::new("a".to_string(), ModuleKind::Js, None);
    let module_b = Module::new("b".to_string(), ModuleKind::Js, None);
    let module_c = Module::new("c".to_string(), ModuleKind::Js, None);
    let module_d = Module::new("d".to_string(), ModuleKind::Js, None);
    let module_e = Module::new("e".to_string(), ModuleKind::Js, None);
    let module_f = Module::new("f".to_string(), ModuleKind::Js, None);

    module_graph.add_module(module_a);
    module_graph.add_module(module_b);
    module_graph.add_module(module_c);
    module_graph.add_module(module_d);
    module_graph.add_module(module_e);
    module_graph.add_module(module_f);

    module_graph
      .add_edge(
        "a",
        "c",
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "c".to_string(),
          order: 0,
        },
      )
      .unwrap();

    module_graph
      .add_edge(
        "a",
        "d",
        ModuleGraphEdge {
          kind: ResolveKind::DynamicImport,
          source: "d".to_string(),
          order: 1,
        },
      )
      .unwrap();

    module_graph
      .add_edge(
        "c",
        "f",
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "f".to_string(),
          order: 0,
        },
      )
      .unwrap();

    module_graph
      .add_edge(
        "d",
        "f",
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "f".to_string(),
          order: 0,
        },
      )
      .unwrap();

    module_graph
      .add_edge(
        "b",
        "e",
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "e".to_string(),
          order: 0,
        },
      )
      .unwrap();

    module_graph
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_module_graph() {
    let module_graph = ModuleGraph::mock_module_graph();

    let a_deps = module_graph.dependencies("a").unwrap();

    assert_eq!(a_deps.len(), 2);

    assert_eq!(
      a_deps.get(0).unwrap(),
      &(
        "c".to_string(),
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "c".to_string(),
          order: 0
        }
      )
    );

    assert_eq!(
      a_deps.get(1).unwrap(),
      &(
        "d".to_string(),
        ModuleGraphEdge {
          kind: ResolveKind::DynamicImport,
          source: "d".to_string(),
          order: 1
        }
      )
    );

    let b_deps = module_graph.dependencies("b").unwrap();

    assert_eq!(b_deps.len(), 1);
    assert_eq!(
      b_deps.get(0).unwrap(),
      &(
        "e".to_string(),
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "e".to_string(),
          order: 0
        }
      )
    );

    let c_deps = module_graph.dependencies("c").unwrap();

    assert_eq!(c_deps.len(), 1);
    assert_eq!(
      c_deps.get(0).unwrap(),
      &(
        "f".to_string(),
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "f".to_string(),
          order: 0
        }
      )
    );

    let d_deps = module_graph.dependencies("d").unwrap();

    assert_eq!(d_deps.len(), 1);
    assert_eq!(
      d_deps.get(0).unwrap(),
      &(
        "f".to_string(),
        ModuleGraphEdge {
          kind: ResolveKind::Import,
          source: "f".to_string(),
          order: 0
        }
      )
    );

    let f_deps = module_graph.dependencies("f").unwrap();

    assert_eq!(f_deps.len(), 0);
  }
}
