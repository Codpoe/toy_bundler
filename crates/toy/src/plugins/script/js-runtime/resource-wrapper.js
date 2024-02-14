(function (modules) {
  var globalObject =
    typeof window !== 'undefined' && window ||
    typeof self !== 'undefined' && self ||
    typeof global !== 'undefined' && global ||
    Function("return this")();
  
  globalObject.__toyModuleSystem__.register(modules);
})(modules);