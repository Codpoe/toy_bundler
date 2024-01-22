use std::collections::{HashMap, HashSet};

use petgraph::{
  stable_graph::{NodeIndex, StableDiGraph},
  Direction,
};

use crate::error::{CompilationError, Result};

use super::{module::Module, ResolveKind};

#[derive(Clone)]
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
  entries: HashSet<String>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      graph: StableDiGraph::new(),
      id_to_index: HashMap::new(),
      entries: HashSet::new(),
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
}
