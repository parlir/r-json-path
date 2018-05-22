/* tslint:disable */
var wasm;

const TextEncoder = require('util').TextEncoder;

let cachedEncoder = new TextEncoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null ||
        cachegetUint8Memory.buffer !== wasm.memory.buffer)
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    return cachegetUint8Memory;
}

function passStringToWasm(arg) {

    const buf = cachedEncoder.encode(arg);
    const ptr = wasm.__wbindgen_malloc(buf.length);
    getUint8Memory().set(buf, ptr);
    return [ptr, buf.length];
}

const TextDecoder = require('util').TextDecoder;

let cachedDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

let cachedGlobalArgumentPtr = null;
function globalArgumentPtr() {
    if (cachedGlobalArgumentPtr === null)
        cachedGlobalArgumentPtr = wasm.__wbindgen_global_argument_ptr();
    return cachedGlobalArgumentPtr;
}

let cachegetUint32Memory = null;
function getUint32Memory() {
    if (cachegetUint32Memory === null ||
        cachegetUint32Memory.buffer !== wasm.memory.buffer)
        cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
    return cachegetUint32Memory;
}

module.exports.json_path = function(arg0, arg1) {
    const [ptr0, len0] = passStringToWasm(arg0);
    const [ptr1, len1] = passStringToWasm(arg1);
    const retptr = globalArgumentPtr();
    try {
        wasm.json_path(retptr, ptr0, len0, ptr1, len1);
        const mem = getUint32Memory();
        const ptr = mem[retptr / 4];
        const len = mem[retptr / 4 + 1];
        const realRet = getStringFromWasm(ptr, len);
        wasm.__wbindgen_free(ptr, len * 1);
        return realRet;
    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);
        wasm.__wbindgen_free(ptr1, len1 * 1);
    }
};

let slab = [];

let slab_next = 0;

function addHeapObject(obj) {
    if (slab_next === slab.length)
        slab.push(slab.length + 1);
    const idx = slab_next;
    const next = slab[idx];

    slab_next = next;

    slab[idx] = { obj, cnt: 1 };
    return idx << 1;
}

let stack = [];

function getObject(idx) {
    if ((idx & 1) === 1) {
        return stack[idx >> 1];
    } else {
        const val = slab[idx >> 1];

    return val.obj;

    }
}

module.exports.__wbindgen_object_clone_ref = function(idx) {
    // If this object is on the stack promote it to the heap.
    if ((idx & 1) === 1)
        return addHeapObject(getObject(idx));

    // Otherwise if the object is on the heap just bump the
    // refcount and move on
    const val = slab[idx >> 1];
    val.cnt += 1;
    return idx;
};

function dropRef(idx) {

    let obj = slab[idx >> 1];

    obj.cnt -= 1;
    if (obj.cnt > 0)
        return;

    // If we hit 0 then free up our space in the slab
    slab[idx >> 1] = slab_next;
    slab_next = idx >> 1;
}

module.exports.__wbindgen_object_drop_ref = function(i) { dropRef(i); };

module.exports.__wbindgen_string_new = function(p, l) {
    return addHeapObject(getStringFromWasm(p, l));
};

module.exports.__wbindgen_number_new = function(i) { return addHeapObject(i); };

module.exports.__wbindgen_number_get = function(n, invalid) {
    let obj = getObject(n);
    if (typeof(obj) === 'number')
        return obj;
    getUint8Memory()[invalid] = 1;
    return 0;
};

module.exports.__wbindgen_undefined_new = function() { return addHeapObject(undefined); };

module.exports.__wbindgen_null_new = function() {
    return addHeapObject(null);
};

module.exports.__wbindgen_is_null = function(idx) {
    return getObject(idx) === null ? 1 : 0;
};

module.exports.__wbindgen_is_undefined = function(idx) {
    return getObject(idx) === undefined ? 1 : 0;
};

module.exports.__wbindgen_boolean_new = function(v) {
    return addHeapObject(v === 1);
};

module.exports.__wbindgen_boolean_get = function(i) {
    let v = getObject(i);
    if (typeof(v) === 'boolean') {
        return v ? 1 : 0;
    } else {
        return 2;
    }
};

module.exports.__wbindgen_symbol_new = function(ptr, len) {
    let a;
    if (ptr === 0) {
        a = Symbol();
    } else {
        a = Symbol(getStringFromWasm(ptr, len));
    }
    return addHeapObject(a);
};

module.exports.__wbindgen_is_symbol = function(i) {
    return typeof(getObject(i)) === 'symbol' ? 1 : 0;
};

module.exports.__wbindgen_string_get = function(i, len_ptr) {
    let obj = getObject(i);
    if (typeof(obj) !== 'string')
        return 0;
    const [ptr, len] = passStringToWasm(obj);
    getUint32Memory()[len_ptr / 4] = len;
    return ptr;
};

module.exports.__wbindgen_throw = function(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
};

wasm = require('./json_path_bg');