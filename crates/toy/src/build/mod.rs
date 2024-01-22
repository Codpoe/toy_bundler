use rayon::ThreadPool;
use std::sync::{
  mpsc::{channel, Sender},
  Arc,
};

use crate::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ResolveKind,
  plugin::ResolveHookParams,
  Compiler,
};

impl Compiler {
  pub(crate) fn build(&mut self) -> Result<()> {
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
      let resolve_result = call_and_catch_error!(resolve, &resolve_hook_params, &context);
      println!(">>> {resolve_result:?}");

      // load

      // transform

      // parse

      // analyze deps

      // build_module recursively
    });
  }
}
