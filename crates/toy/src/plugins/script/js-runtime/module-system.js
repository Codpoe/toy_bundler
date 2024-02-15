(function bootstrap(modules, entryId) {
  var globalObject =
    typeof window !== 'undefined' && window ||
    typeof self !== 'undefined' && self ||
    typeof global !== 'undefined' && global ||
    Function("return this")();

  globalObject.__toyModuleSystem__ = globalObject.__toyModuleSystem__ || (function () {
    var cache = {};
    var modules = {};

    function register(_modules) {
      Object.assign(modules, _modules);
    }

    function require(id) {
      if (cache[id]) {
        return cache[id];
      }

      var moduleFactory = modules[id];

      if (!moduleFactory) {
        throw new Error('Module not found: ' + id);
      }

      var module = {
        exports: {},
      };

      moduleFactory(module, require, dynamicRequire);
      cache[id] = module.exports;

      return module.exports;
    }

    function dynamicRequire() {}

    return {
      register,
      require,
    }
  })();

  globalObject.__toyModuleSystem__.register(modules);
  globalObject.__toyModuleSystem__.require(entryId);
})(modules, entryId);