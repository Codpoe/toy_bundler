use rayon::ThreadPool;
use std::sync::{
  mpsc::{channel, Sender},
  Arc,
};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraphEdge, ResolveKind},
  plugin::{
    AnalyzeDepsHookParams, LoadHookParams, ParseHookParams, ResolveHookParams, TransformHookParams,
  },
  Compiler,
};

impl Compiler {
  pub(crate) fn build(&mut self) -> Result<()> {
    self.context.plugin_container.build_start(&self.context)?;

    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let (err_sender, err_receiver) = channel::<CompilationError>();

    self
      .context
      .config
      .input
      .values()
      .enumerate()
      .for_each(|(order, source)| {
        Self::build_module(
          thread_pool.clone(),
          err_sender.clone(),
          order,
          ResolveHookParams {
            source: source.clone(),
            importer: None,
            kind: ResolveKind::Entry,
          },
          self.context.clone(),
        )
      });

    drop(err_sender);

    if let Ok(err) = err_receiver.recv() {
      return Err(err);
    }

    self.context.plugin_container.build_end(&self.context)
  }

  fn build_module(
    thread_pool: Arc<ThreadPool>,
    err_sender: Sender<CompilationError>,
    order: usize,
    resolve_hook_params: ResolveHookParams,
    context: Arc<CompilationContext>,
  ) {
    let c_thread_pool = thread_pool.clone();
    let context = context.clone();

    thread_pool.spawn(move || {
      macro_rules! call_and_catch_error {
        ($func:ident, $($arg:expr),*) => {
          match context.plugin_container.$func($($arg),*) {
            Ok(result) => result,
            Err(error) => {
              err_sender.send(error).expect("send error to main thread failed");
              return;
            }
          }
        };
      }

      // resolve
      let resolve_result = call_and_catch_error!(resolve, &resolve_hook_params, &context).unwrap();
      println!(">>> resolve_result: {resolve_result:#?}");

      // load
      let load_params = LoadHookParams {
        id: resolve_result.id.clone(),
        query: resolve_result.query.clone(),
      };
      let load_result = call_and_catch_error!(load, &load_params, &context).unwrap();
      println!(">>> load_result: {load_result:#?}");

      // transform
      let transform_params = TransformHookParams {
        id: resolve_result.id.clone(),
        query: resolve_result.query.clone(),
        content: load_result.content,
        module_kind: load_result.module_kind,
      };
      let transform_result = call_and_catch_error!(transform, transform_params, &context);
      println!(">>> transform_result: {transform_result:#?}");

      // parse
      let parse_params = ParseHookParams {
        id: resolve_result.id.clone(),
        query: resolve_result.query.clone(),
        content: transform_result.content,
        module_kind: transform_result.module_kind,
      };
      let module = call_and_catch_error!(parse, &parse_params, &context).unwrap();

      // analyze deps
      let mut analyze_deps_params = AnalyzeDepsHookParams {
        module: &module,
        deps: vec![],
      };
      call_and_catch_error!(analyze_deps, &mut analyze_deps_params, &context);

      let deps = analyze_deps_params.deps;
      println!(">>> analyze_deps {:?} -> {:#?}", resolve_result.id, deps);

      // build module_graph
      let module_id = module.id.clone();
      let mut module_graph = context.module_graph.write().unwrap();

      module_graph.add_module(module);

      if matches!(resolve_hook_params.kind, ResolveKind::Entry) {
        module_graph.entries.insert(module_id.clone());
      }

      if let Some(importer) = resolve_hook_params.importer {
        module_graph
          .add_edge(
            &importer,
            &module_id,
            ModuleGraphEdge {
              kind: resolve_hook_params.kind.clone(),
              source: resolve_hook_params.source.clone(),
              order,
            },
          )
          .unwrap()
      }

      drop(module_graph);

      // build_module recursively
      for (order, dep) in deps.iter().enumerate() {
        Self::build_module(
          c_thread_pool.clone(),
          err_sender.clone(),
          order,
          ResolveHookParams {
            source: dep.source.clone(),
            importer: Some(module_id.clone()),
            kind: dep.resolve_kind.clone(),
          },
          Arc::clone(&context),
        );
      }
    });
  }
}
