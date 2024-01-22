pub mod module;
pub mod module_graph;
pub mod module_group;

#[derive(Debug, Clone)]
pub enum ResolveKind {
  /// entry input in the config
  Entry,
  /// static import, e.g. `import a from './a'`
  Import,
  /// dynamic import, e.g. `import('./a').then(module => console.log(module))`
  DynamicImport,
  /// cjs require, e.g. `require('./a')`
  Require,
  /// @import of css, e.g. @import './a.css'
  CssAtImport,
  /// url() of css, e.g. url('./a.png')
  CssUrl,
  /// `<script src="./index.html" />` of html
  ScriptSrc,
  /// `<link href="index.css" />` of html
  LinkHref,
  /// Custom ResolveKind, e.g. `const worker = new Worker(new Url("worker.js"))` of a web worker
  Custom(String),
}
