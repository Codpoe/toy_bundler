(function bootstrap(modules, entryId) {
  const globalObject =
    typeof window !== 'undefined' && window ||
    typeof self !== 'undefined' && self ||
    typeof global !== 'undefined' && global ||
    Function("return this")();

  globalObject.__toyModuleSystem__ = globalObject.__toyModuleSystem__ || (function () {
    const cache = {};
    const modules = {};
    // TODO: 注入动态加载的资源 manifest
    const dynamicIdToResource = {};

    function register(_modules) {
      Object.assign(modules, _modules);
    }

    function require(id) {
      if (cache[id]) {
        return cache[id];
      }

      const moduleFactory = modules[id];

      if (!moduleFactory) {
        throw new Error('Module not found: ' + id);
      }

      const module = {
        exports: {},
      };

      moduleFactory(module, require, dynamicRequire);
      cache[id] = module.exports;

      return module.exports;
    }

    function dynamicRequire(id) {
      return new Promise((resolve, reject) => {
        if (modules[id]) {
          return resolve(require(id));
        }
    
        const resource = dynamicIdToResource[id];
    
        if (!resource) {
          return reject(new Error('Module not found: ' + id));
        }
    
        const scriptEl = document.createElement('script');
  
        scriptEl.src = resource;
        scriptEl.onload = function () {
          resolve(require(id));
        };
        scriptEl.onerror = function () {
          reject(new Error('Module load failed: ' + id));
        };
        document.head.appendChild(scriptEl);
      });
    }

    return {
      register,
      require,
      dynamicRequire,
    }
  })();

  globalObject.__toyModuleSystem__.register(modules);

  if (entryId) {
    globalObject.__toyModuleSystem__.require(entryId);
  }
})(modules, entryId);