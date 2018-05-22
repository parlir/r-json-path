let imports = {};
imports['./json_path'] = require('./json_path');

            const join = require('path').join;
            const bytes = require('fs').readFileSync(join(__dirname, 'json_path_bg.wasm'));
            const wasmModule = new WebAssembly.Module(bytes);
            const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
            module.exports = wasmInstance.exports;
        