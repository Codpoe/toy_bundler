use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module::{Module, ModuleKind},
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupMap},
  },
  resource::{
    resource::{Resource, ResourceMap},
    resource_pot::ResourcePot,
  },
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::Arc};

use super::{
  AnalyzeDepsHookParams, LoadHookParams, LoadHookResult, ParseHookParams, Plugin,
  ResolveHookParams, ResolveHookResult, TransformHookParams,
};

#[derive(Debug)]
pub struct PluginContainerTransformHookResult {
  content: String,
  module_kind: ModuleKind,
  source_map_chain: Vec<String>,
}

pub struct PluginContainer {
  plugins: Vec<Arc<dyn Plugin>>,
}

macro_rules! hook_first {
  ($func:ident, $params:ident: $ty:ty, $ret_ty:ty) => {
    pub fn $func(
      &self,
      $params: $ty,
      context: &Arc<CompilationContext>,
    ) -> Result<Option<$ret_ty>> {
      for plugin in &self.plugins {
        let ret = plugin.$func($params, context)?;

        if ret.is_some() {
          return Ok(ret);
        }
      }
      Ok(None)
    }
  };
}

macro_rules! hook_serial {
  ($func:ident, $param:ident: $ty:ty) => {
    pub fn $func(&self, $param: $ty, context: &Arc<CompilationContext>) -> Result<()> {
      for plugin in &self.plugins {
        plugin.$func($param, context)?;
      }
      Ok(())
    }
  };
}

macro_rules! hook_parallel {
  ($func:ident) => {
    pub fn $func(&self, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| plugin.$func(context).map(|_| ()))
    }
  };
  ($func:ident, $($params:ident: $ty:ty),+) => {
    pub fn $func(&self, $($params: $ty),+, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| plugin.$func($($params),+, context).map(|_| ()))
    }
  };
}

impl PluginContainer {
  pub fn new(plugins: Vec<Arc<dyn Plugin>>) -> Self {
    Self { plugins }
  }

  pub fn config(&self, config: &mut Config) -> Result<()> {
    for plugin in &self.plugins {
      plugin.config(config)?;
    }
    Ok(())
  }

  hook_parallel!(build_start);

  hook_first!(resolve, params: &ResolveHookParams, ResolveHookResult);

  hook_first!(load, params: &LoadHookParams, LoadHookResult);

  pub fn transform(
    &self,
    mut params: TransformHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<PluginContainerTransformHookResult> {
    let mut ret = PluginContainerTransformHookResult {
      content: params.content.clone(),
      module_kind: params.module_kind.clone(),
      source_map_chain: vec![],
    };

    for plugin in &self.plugins {
      if let Some(plugin_ret) = plugin.transform(&params, context)? {
        params.content = plugin_ret.content;

        if let Some(module_kind) = plugin_ret.module_kind {
          params.module_kind = module_kind;
        }

        if let Some(source_map) = plugin_ret.source_map {
          ret.source_map_chain.push(source_map);
        }
      }
    }

    ret.content = params.content;
    ret.module_kind = params.module_kind;

    Ok(ret)
  }

  hook_first!(parse, params: &ParseHookParams, Module);

  hook_serial!(analyze_deps, params: &mut AnalyzeDepsHookParams);

  hook_parallel!(build_end);

  hook_first!(analyze_module_graph, module_graph: &mut ModuleGraph, ModuleGroupMap);

  hook_first!(analyze_module_group, module_group: &mut ModuleGroup, Vec<ResourcePot>);

  hook_serial!(render_resource_pot, resource_pot: &mut ResourcePot);

  hook_first!(generate_resources, resource_pot: &mut ResourcePot, HashMap<String, Resource>);

  hook_serial!(write_resources, resources: &mut ResourceMap);
}
