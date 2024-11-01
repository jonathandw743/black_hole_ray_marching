let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error(`expected a boolean argument, found ${typeof(n)}`);
    }
}

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
}
function __wbg_adapter_26(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha3e46234ee116d53(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_29(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0615fc132acdf266(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_32(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc056286cc62a6825(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_35(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h79708a9fca96418d(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_38(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h614135dbdb1dce7b(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wbg_adapter_41(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h553157214e037656(arg0, arg1);
}

function __wbg_adapter_44(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc831d31bbb76fec1(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_47(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5b463b0d9212a9c6(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_50(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h8c591dee382a3a70(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_53(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc7654f3a9633f329(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_56(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h11f2890944d993db(arg0, arg1, addHeapObject(arg2));
}

export function run() {
    wasm.run();
}

function notDefined(what) { return () => { throw new Error(`${what} is not defined`); }; }

let cachedUint32ArrayMemory0 = null;

function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];

const __wbindgen_enum_GpuAutoLayoutMode = ["auto"];

const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant"];

const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];

const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];

const __wbindgen_enum_GpuBufferMapState = ["unmapped", "pending", "mapped"];

const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];

const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];

const __wbindgen_enum_GpuCompilationMessageType = ["error", "warning", "info"];

const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];

const __wbindgen_enum_GpuDeviceLostReason = ["unknown", "destroyed"];

const __wbindgen_enum_GpuErrorFilter = ["validation", "out-of-memory", "internal"];

const __wbindgen_enum_GpuFeatureName = ["depth-clip-control", "depth32float-stencil8", "texture-compression-bc", "texture-compression-etc2", "texture-compression-astc", "timestamp-query", "indirect-first-instance", "shader-f16", "rg11b10ufloat-renderable", "bgra8unorm-storage", "float32-filterable"];

const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];

const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];

const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];

const __wbindgen_enum_GpuLoadOp = ["load", "clear"];

const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];

const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];

const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];

const __wbindgen_enum_GpuQueryType = ["occlusion", "timestamp"];

const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];

const __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];

const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];

const __wbindgen_enum_GpuStoreOp = ["store", "discard"];

const __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];

const __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];

const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];

const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];

const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];

const __wbindgen_enum_GpuVertexFormat = ["uint8x2", "uint8x4", "sint8x2", "sint8x4", "unorm8x2", "unorm8x4", "snorm8x2", "snorm8x4", "uint16x2", "uint16x4", "sint16x2", "sint16x4", "unorm16x2", "unorm16x4", "snorm16x2", "snorm16x4", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2"];

const __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];

const __wbindgen_enum_PremultiplyAlpha = ["none", "premultiply", "default"];

const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];

const __wbindgen_enum_VisibilityState = ["hidden", "visible"];

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_now_2353d97c748b7a64 = typeof Date.now == 'function' ? Date.now : notDefined('Date.now');
    imports.wbg.__wbg_error_f851667af71bcfc6 = function() { return logError(function (arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    }, arguments) };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() { return logError(function () {
        const ret = new Error();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_animate_b321da85ed3f2b4a = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).animate(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_cancel_cba86749f45f30ae = function() { return logError(function (arg0) {
        getObject(arg0).cancel();
    }, arguments) };
    imports.wbg.__wbg_play_5896e5851ba90aa2 = function() { return logError(function (arg0) {
        getObject(arg0).play();
    }, arguments) };
    imports.wbg.__wbg_Window_bd9ec3fee5f673ee = function() { return logError(function (arg0) {
        const ret = getObject(arg0).Window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestFullscreen_24891df6120b675d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestFullscreen();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_webkitRequestFullscreen_42ba1c34171febc6 = function() { return logError(function (arg0) {
        getObject(arg0).webkitRequestFullscreen();
    }, arguments) };
    imports.wbg.__wbg_webkitFullscreenElement_a02341d57a641b43 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).webkitFullscreenElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_webkitExitFullscreen_77a6c8d07ec6ee46 = function() { return logError(function (arg0) {
        getObject(arg0).webkitExitFullscreen();
    }, arguments) };
    imports.wbg.__wbg_requestFullscreen_8a94df4e7f757077 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestFullscreen;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_prototype_d33365945f23f380 = function() { return logError(function () {
        const ret = ResizeObserverEntry.prototype;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_userAgentData_85a8393570ab7dee = function() { return logError(function (arg0) {
        const ret = getObject(arg0).userAgentData;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_brands_982de08b35281a98 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).brands;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_brand_cdcf0249d44027a8 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).brand;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbg_scheduler_f38a681d98b5a776 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).scheduler;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestIdleCallback_86b728660e0547ef = function() { return logError(function (arg0) {
        const ret = getObject(arg0).requestIdleCallback;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_scheduler_7ccf2d3b362018c4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).scheduler;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_postTask_99464245f349be5a = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).postTask(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_693514e3dcae83e6 = function() { return logError(function (arg0) {
        queueMicrotask(takeObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_offsetX_294898d040917c6b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).offsetX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_offsetY_f484804b7b03dd86 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).offsetY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getCoalescedEvents_85701851c470c4e6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCoalescedEvents;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_performance_a1b8bde2ee512264 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).performance;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_now_abd80e969af37148 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).now();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getPreferredCanvasFormat_d55bc32b5a6b948a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getPreferredCanvasFormat();
        return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 96) - 1;
    }, arguments) };
    imports.wbg.__wbg_requestAdapter_8413757c51a35b1d = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).requestAdapter(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_features_b1971639ec1a77f7 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).features;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_limits_e806d307d42a9dde = function() { return logError(function (arg0) {
        const ret = getObject(arg0).limits;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_queue_e124eaca54d285d4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).queue;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_lost_02e8ddfb37103cc2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).lost;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setonuncapturederror_c702acc9eeeb9613 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onuncapturederror = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_createBindGroup_f93afa3a0a06b10e = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBindGroup(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createBindGroupLayout_4243a95be946d48a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createBuffer_44406243485760b1 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createBuffer(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createCommandEncoder_c7eddb5143f91992 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createComputePipeline_fb60500f9a96e290 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createComputePipeline(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createPipelineLayout_bcb406883550f9cc = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createQuerySet_4040f9ea5a2ac03c = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createQuerySet(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderBundleEncoder_d9644450ab4cad8f = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createRenderPipeline_7ca396c186d8d06a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createSampler_ed81ff565caa903a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createSampler(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createShaderModule_cda89eb5c1073627 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createShaderModule(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createTexture_06106f81b60e5462 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createTexture(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_destroy_2a8c41712abac4cb = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_popErrorScope_6d6b4abc95412374 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).popErrorScope();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_pushErrorScope_3dc565fa86fee870 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).pushErrorScope(__wbindgen_enum_GpuErrorFilter[arg1]);
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuOutOfMemoryError_658135cd3b3f08e2 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof GPUOutOfMemoryError;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_copyExternalImageToTexture_e192d56d70996ad4 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_submit_4283b63806c5d15e = function() { return logError(function (arg0, arg1) {
        getObject(arg0).submit(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_writeBuffer_6ce87bc6ff22a2b5 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_writeTexture_3708ced0dd386721 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
    }, arguments) };
    imports.wbg.__wbg_Window_4d1f8d969d639a22 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).Window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_WorkerGlobalScope_c4f12290f7d2efed = function() { return logError(function (arg0) {
        const ret = getObject(arg0).WorkerGlobalScope;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_message_0ff806941d54e1d2 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).message;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_type_c3e79de7c41f03c2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).type;
        return (__wbindgen_enum_GpuCompilationMessageType.indexOf(ret) + 1 || 4) - 1;
    }, arguments) };
    imports.wbg.__wbg_lineNum_06a4c70c1027df81 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).lineNum;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_offset_47f9a19926637c8e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).offset;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_length_ff62902e8840f82f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).length;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_dispatchWorkgroups_f0fd90dcd4a506fa = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_dispatchWorkgroupsIndirect_567a84763f6a0b87 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_end_bbe499813ce72830 = function() { return logError(function (arg0) {
        getObject(arg0).end();
    }, arguments) };
    imports.wbg.__wbg_setPipeline_4d0e04e7370f0e2e = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_48300d51a3d74853 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_d79f4f1d5e43c06f = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuDeviceLostInfo_c7232ceb822b15d6 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof GPUDeviceLostInfo;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_reason_436ee862de561851 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).reason;
        return (__wbindgen_enum_GpuDeviceLostReason.indexOf(ret) + 1 || 3) - 1;
    }, arguments) };
    imports.wbg.__wbg_message_54cb97c0fd1579bf = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).message;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_end_c97b7dbccda72e72 = function() { return logError(function (arg0) {
        getObject(arg0).end();
    }, arguments) };
    imports.wbg.__wbg_executeBundles_0f6b9b3accb5b6a7 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).executeBundles(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setBlendConstant_fd172910ef2cc0c8 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setBlendConstant(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setScissorRect_915b4534e3936f28 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setStencilReference_e2bb05496423e92e = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setStencilReference(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setViewport_aff318ede051c64e = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_da48569994113ec3 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_1c3dd07b998fa943 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_draw_a3e2be7a25d4af68 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexed_f219cccc74b869c5 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexedIndirect_6839c0505e2eed2e = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_drawIndirect_23fc0a72c5f1b993 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_1dc175abfd5d9be9 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_a0fcb26f210351b7 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_setPipeline_8f2f5c316ddb7f68 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_c347f9618d3f056a = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_40da6368898587db = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_getCompilationInfo_adcb4d74ed54d1f9 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCompilationInfo();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension1D_71c238385d79f287 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension1D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension2D_ce910a0ea6c7213b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension2D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureDimension3D_76032d2a97af63ac = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureDimension3D;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxTextureArrayLayers_b561668f7e1ebacc = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxTextureArrayLayers;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBindGroups_d2b688642140a1bb = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBindGroups;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBindingsPerBindGroup_a3e9e404dd893c83 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBindingsPerBindGroup;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxDynamicUniformBuffersPerPipelineLayout_98a8fbca367148bf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxDynamicStorageBuffersPerPipelineLayout_0dec6aea74b472ad = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxSampledTexturesPerShaderStage_7a0712465c0a12b4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxSamplersPerShaderStage_6716e9792fc2a833 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxSamplersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageBuffersPerShaderStage_640d34738978a4ff = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageTexturesPerShaderStage_6614a1e40f7e2827 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxUniformBuffersPerShaderStage_1ff2f3c6468374ae = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxUniformBufferBindingSize_8830a8df4f730637 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxUniformBufferBindingSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxStorageBufferBindingSize_10b6eb49372335bc = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxStorageBufferBindingSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_minUniformBufferOffsetAlignment_0168a0d08b19afbe = function() { return logError(function (arg0) {
        const ret = getObject(arg0).minUniformBufferOffsetAlignment;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_minStorageBufferOffsetAlignment_3b63a59f37f275f8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).minStorageBufferOffsetAlignment;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexBuffers_9f97f2a89863a431 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexBuffers;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxBufferSize_1c8b836056558ebf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxBufferSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexAttributes_cff466bbace9aa7c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexAttributes;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxVertexBufferArrayStride_fb650714a5bd0e68 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxVertexBufferArrayStride;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxInterStageShaderComponents_db659eaa3b248a74 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxInterStageShaderComponents;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxColorAttachments_e821b856b5cba24e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxColorAttachments;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxColorAttachmentBytesPerSample_ab770042dd82a5bf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxColorAttachmentBytesPerSample;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupStorageSize_e6773eb1cbfa7a83 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeInvocationsPerWorkgroup_4ed447998b195973 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeX_de94f4925b26c73c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeX;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeY_cb75de6b450c8915 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeY;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupSizeZ_6277d18773d70891 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_maxComputeWorkgroupsPerDimension_baef21641827881d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuValidationError_05482398d349fd2d = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof GPUValidationError;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        const ret = typeof(val) === 'object' && val !== null;
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbg_instanceof_GpuCanvasContext_1eacd2a8c6b36ada = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof GPUCanvasContext;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_configure_48cfbf148a9998c2 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).configure(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_getCurrentTexture_1c8e29bec577927d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCurrentTexture();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_label_81cb6c4ebcba5f4d = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).label;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_beginComputePass_df50d9ddd5f32a63 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).beginComputePass(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_beginRenderPass_14284a54cee2063b = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).beginRenderPass(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_clearBuffer_a5ccb106665ad51e = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).clearBuffer(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_clearBuffer_f06a69a0aa134d24 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
    }, arguments) };
    imports.wbg.__wbg_copyBufferToBuffer_f0736fef84f76be5 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_copyBufferToTexture_aedde01ad3786b4f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_copyTextureToBuffer_268207d3e09dfa81 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_copyTextureToTexture_7ea3d6de0a82ce7f = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_finish_7ad9d3e23124bbc6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).finish();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_finish_78696a2f194fbb7a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).finish(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_resolveQuerySet_7354946ea63dacbb = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_finish_5be91110098e071c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).finish();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_finish_667443ed0047f53a = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).finish(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_de4812744c6ebb6c = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_setBindGroup_92581920e209bf52 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_draw_29abcb466fee48b4 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexed_34b06707991ddaf7 = function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_drawIndexedIndirect_4b7b51fa979657ca = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_drawIndirect_0054fe754e8e46e9 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).drawIndirect(getObject(arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_91b6f5eb1a43df9b = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3);
    }, arguments) };
    imports.wbg.__wbg_setIndexBuffer_5bce79843be8653d = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_setPipeline_6174c2e8900fe24a = function() { return logError(function (arg0, arg1) {
        getObject(arg0).setPipeline(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_d9b48c3489dcfa22 = function() { return logError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_setVertexBuffer_330ab505b9dfc64b = function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_getBindGroupLayout_1490d5a61f4fd56b = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createView_87e589e1574ba76c = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).createView(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_destroy_b040948312c539a9 = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_messages_6833dfd0ae6a0a7c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).messages;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_GpuAdapter_ba82c448cfa55608 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof GPUAdapter;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_features_e7f12cb6c5258238 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).features;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_limits_622a6ae19a037dbf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).limits;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestDevice_1c8e4f0fe8729328 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).requestDevice(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_size_61d4fa05868b79cd = function() { return logError(function (arg0) {
        const ret = getObject(arg0).size;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_usage_5043ac06189fbe53 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).usage;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_destroy_387cb19081689594 = function() { return logError(function (arg0) {
        getObject(arg0).destroy();
    }, arguments) };
    imports.wbg.__wbg_getMappedRange_08e71df297c66a50 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getMappedRange(arg1, arg2);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_mapAsync_98ce4986e2f6d4af = function() { return logError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_unmap_efca7885e5daff78 = function() { return logError(function (arg0) {
        getObject(arg0).unmap();
    }, arguments) };
    imports.wbg.__wbg_getBindGroupLayout_0194b7a790ac805d = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_has_14b751afdcf0a341 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_error_520ca6f621497012 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).error;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_message_4bd9ef09b3092122 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).message;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_gpu_7d756a02ad45027d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).gpu;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'function';
        _assertBoolean(ret);
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_481971b0d87f3dd4 = function() { return logError(function (arg0) {
        queueMicrotask(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).queueMicrotask;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Window_f401953a2cf86220 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_document_5100775d18896c16 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_navigator_6c8fa55c5cc8796e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_devicePixelRatio_efc553b59506f64c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).devicePixelRatio;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cancelIdleCallback_3a36cf77475b492b = function() { return logError(function (arg0, arg1) {
        getObject(arg0).cancelIdleCallback(arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_getComputedStyle_078292ffe423aded = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).getComputedStyle(getObject(arg1));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_matchMedia_66bb21e3ef19270c = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_open_cc82b8aaf0c296c1 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestIdleCallback_cee8e1d6bdcfae9e = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).requestIdleCallback(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cancelAnimationFrame_111532f326e480af = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).cancelAnimationFrame(arg1);
    }, arguments) };
    imports.wbg.__wbg_requestAnimationFrame_549258cfa66011f0 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clearTimeout_ba63ae54a36e111e = function() { return logError(function (arg0, arg1) {
        getObject(arg0).clearTimeout(arg1);
    }, arguments) };
    imports.wbg.__wbg_setTimeout_d2b9a986d10a6182 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).setTimeout(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_c172d5704ef82276 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_body_edb1908d3ceff3a1 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).body;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_visibilityState_990071edf70b1c55 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).visibilityState;
        return (__wbindgen_enum_VisibilityState.indexOf(ret) + 1 || 3) - 1;
    }, arguments) };
    imports.wbg.__wbg_activeElement_fa7feca08f5028c0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).activeElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_fullscreenElement_1bef71098bd8dfde = function() { return logError(function (arg0) {
        const ret = getObject(arg0).fullscreenElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createElement_8bae7856a4bb7411 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_exitFullscreen_5679ad2b002921bd = function() { return logError(function (arg0) {
        getObject(arg0).exitFullscreen();
    }, arguments) };
    imports.wbg.__wbg_getElementById_c369ff43f0db99cf = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_querySelectorAll_4e0fcdb64cda2cd5 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setAttribute_3c9f6c303b696daa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setPointerCapture_0fdaad7a916c8486 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).setPointerCapture(arg1);
    }, arguments) };
    imports.wbg.__wbg_newwithstrsequenceandoptions_ce1f1ca2d522b8aa = function() { return handleError(function (arg0, arg1) {
        const ret = new Blob(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getPropertyValue_fa32ee1811f224cb = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_removeProperty_fa6d48e2923dcfac = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_setProperty_ea7d15a2b591aa97 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_altKey_2e6c34c37088d8b1 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).altKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_ctrlKey_bb5b6fef87339703 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).ctrlKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shiftKey_5911baf439ab232b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).shiftKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_metaKey_6bf4ae4e83a11278 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).metaKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_location_f7b033ddfc516739 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).location;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_repeat_f64b916c6eed0685 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).repeat;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_key_dccf9e8aa1315a8e = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).key;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_code_3b0c3912a2351163 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).code;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_setonmessage_93bdba94dcd46c04 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).onmessage = getObject(arg1);
    }, arguments) };
    imports.wbg.__wbg_close_a5883ed21dc3d115 = function() { return logError(function (arg0) {
        getObject(arg0).close();
    }, arguments) };
    imports.wbg.__wbg_postMessage_fbddfe9314af804e = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).postMessage(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_start_5a293222bc398f51 = function() { return logError(function (arg0) {
        getObject(arg0).start();
    }, arguments) };
    imports.wbg.__wbg_persisted_cbb7e3c657029516 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).persisted;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_61d4f20a1c08a45c = function() { return handleError(function (arg0) {
        const ret = new ResizeObserver(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_disconnect_6675f32e2ae8deb7 = function() { return logError(function (arg0) {
        getObject(arg0).disconnect();
    }, arguments) };
    imports.wbg.__wbg_observe_a79646ce7bb08cb8 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).observe(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_observe_dc0ebcd59ee7cd17 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).observe(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_unobserve_55c93518cad6ac06 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).unobserve(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_contentRect_bce644376332c7a5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).contentRect;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_devicePixelContentBoxSize_d5bcdcd5e96671f3 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).devicePixelContentBoxSize;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_createObjectURL_ad8244759309f204 = function() { return handleError(function (arg0, arg1) {
        const ret = URL.createObjectURL(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_revokeObjectURL_16a2051ee9d99da9 = function() { return handleError(function (arg0, arg1) {
        URL.revokeObjectURL(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_style_c3fc3dd146182a2d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).style;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_focus_39d4b8ba8ff9df14 = function() { return handleError(function (arg0) {
        getObject(arg0).focus();
    }, arguments) };
    imports.wbg.__wbg_navigator_56803b85352a0575 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_debug_5fb96680aecf5dc8 = function() { return logError(function (arg0) {
        console.debug(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_error_8e3928cfb8a43e2b = function() { return logError(function (arg0) {
        console.error(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_error_6e987ee48d9fdf45 = function() { return logError(function (arg0, arg1) {
        console.error(getObject(arg0), getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_info_530a29cb2e4e3304 = function() { return logError(function (arg0) {
        console.info(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_log_5bb5f88f245d7762 = function() { return logError(function (arg0) {
        console.log(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_warn_63bbae1730aead09 = function() { return logError(function (arg0) {
        console.warn(getObject(arg0));
    }, arguments) };
    imports.wbg.__wbg_signal_a61f78a3478fd9bc = function() { return logError(function (arg0) {
        const ret = getObject(arg0).signal;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_0d76b0581eca6298 = function() { return handleError(function () {
        const ret = new AbortController();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_abort_2aa7521d5690750e = function() { return logError(function (arg0) {
        getObject(arg0).abort();
    }, arguments) };
    imports.wbg.__wbg_width_1e8430024cb82aba = function() { return logError(function (arg0) {
        const ret = getObject(arg0).width;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_height_0c1394f089d7bb71 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).height;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_preventDefault_b1a4aafc79409429 = function() { return logError(function (arg0) {
        getObject(arg0).preventDefault();
    }, arguments) };
    imports.wbg.__wbg_addEventListener_53b787075bd5e003 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_removeEventListener_92cb9b3943463338 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_setwidth_080107476e633963 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_setheight_dc240617639f1f51 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_getContext_df50fa48a8876636 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_4e95a9abecc83cd4 = function() { return handleError(function (arg0) {
        const ret = new IntersectionObserver(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_disconnect_e694940ce6d0ef91 = function() { return logError(function (arg0) {
        getObject(arg0).disconnect();
    }, arguments) };
    imports.wbg.__wbg_observe_538a6d1df0deb993 = function() { return logError(function (arg0, arg1) {
        getObject(arg0).observe(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_port1_d51a1bd2c33125d0 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).port1;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_port2_f522a81e92362e7e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).port2;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_34615e164dc78975 = function() { return handleError(function () {
        const ret = new MessageChannel();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_userAgent_e94c7cbcdac01fea = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg1).userAgent;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_inlineSize_ff0e40258cefeba2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).inlineSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_blockSize_73f4e5608c08713d = function() { return logError(function (arg0) {
        const ret = getObject(arg0).blockSize;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_d1187ae36d662ef9 = function() { return handleError(function (arg0, arg1) {
        const ret = new Worker(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_postMessage_d8e032ebcfacd008 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).postMessage(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_isIntersecting_082397a1d66e2e35 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).isIntersecting;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_media_bcef0e2ec4383569 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).media;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_matches_e14ed9ff8291cf24 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).matches;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addListener_143ad0a501fabc3a = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).addListener(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_removeListener_46f3ee00c5b95320 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).removeListener(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_ctrlKey_008695ce60a588f5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).ctrlKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shiftKey_1e76dbfcdd36a4b4 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).shiftKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_altKey_07da841b54bd3ed6 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).altKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_metaKey_86bfd3b0d3a8083f = function() { return logError(function (arg0) {
        const ret = getObject(arg0).metaKey;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_button_367cdc7303e3cf9b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).button;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_buttons_d004fa75ac704227 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buttons;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_movementX_b800a0cacd14d9bf = function() { return logError(function (arg0) {
        const ret = getObject(arg0).movementX;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_movementY_7907e03eb8c0ea1e = function() { return logError(function (arg0) {
        const ret = getObject(arg0).movementY;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setwidth_83d936c4b04dcbec = function() { return logError(function (arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_setheight_6025ba0d58e6cc8c = function() { return logError(function (arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_getContext_c102f659d540d068 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_deltaX_206576827ededbe5 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deltaY_032e327e216f2b2b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deltaMode_294b2eaf54047265 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).deltaMode;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_appendChild_580ccb11a660db68 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_contains_fdfd1dc667f36695 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).contains(getObject(arg1));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_get_8cd5eba00ab6304f = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_pointerId_e030fa156647fedd = function() { return logError(function (arg0) {
        const ret = getObject(arg0).pointerId;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pressure_99cd07399f942a7c = function() { return logError(function (arg0) {
        const ret = getObject(arg0).pressure;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pointerType_0f2f0383406aa7fa = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg1).pointerType;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getCoalescedEvents_14b443b6f75837a2 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).getCoalescedEvents();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_16b304a2cfa7ff4a = function() { return logError(function () {
        const ret = new Array();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_get_bd8e338fbd5f5cc8 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_length_cd7af8117672b8b8 = function() { return logError(function (arg0) {
        const ret = getObject(arg0).length;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_of_4a2b313a453ec059 = function() { return logError(function (arg0) {
        const ret = Array.of(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_of_647f9238b4d5407a = function() { return logError(function (arg0, arg1) {
        const ret = Array.of(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_push_a5b05aedc7234f9f = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).push(getObject(arg1));
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newnoargs_e258087cd0daa0ea = function() { return logError(function (arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_call_27c0f87801dedf93 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_Object_71ca3c0a59266746 = function() { return logError(function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Object;
        } catch (_) {
            result = false;
        }
        const ret = result;
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getOwnPropertyDescriptor_fcb32c9a1f90b136 = function() { return logError(function (arg0, arg1) {
        const ret = Object.getOwnPropertyDescriptor(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_is_010fdc0f4ab96916 = function() { return logError(function (arg0, arg1) {
        const ret = Object.is(getObject(arg0), getObject(arg1));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_72fb9a18b5ae2624 = function() { return logError(function () {
        const ret = new Object();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_valueOf_a0b7c836f68a054b = function() { return logError(function (arg0) {
        const ret = getObject(arg0).valueOf();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_resolve_b0083a7967828ec8 = function() { return logError(function (arg0) {
        const ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_catch_0260e338d10f79ae = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).catch(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_then_0c86a60e8fcfe9f6 = function() { return logError(function (arg0, arg1) {
        const ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_then_a73caa9a87991566 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_d1e6af4856ba331b = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_self_ce0dbfc45cf2f5be = function() { return handleError(function () {
        const ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_c6fb939a7f436783 = function() { return handleError(function () {
        const ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_207b558942527489 = function() { return handleError(function () {
        const ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_63b92bc8671ed464 = function() { return logError(function (arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb = function() { return logError(function (arg0, arg1, arg2) {
        const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_buffer_dd7f74bc60f1faab = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_length_c20a40f15020d68a = function() { return logError(function (arg0) {
        const ret = getObject(arg0).length;
        _assertNum(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_a47bac70306a19a7 = function() { return logError(function (arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_buffer_12d079cc21e14bdb = function() { return logError(function (arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_1f9b04f170055d33 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
        _assertBoolean(ret);
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper19630 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1042, __wbg_adapter_26);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19632 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1040, __wbg_adapter_29);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19634 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1038, __wbg_adapter_32);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19636 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1034, __wbg_adapter_35);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19638 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1048, __wbg_adapter_38);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19640 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1032, __wbg_adapter_41);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19642 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1046, __wbg_adapter_44);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19644 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1044, __wbg_adapter_47);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper19646 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1036, __wbg_adapter_50);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper24724 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1311, __wbg_adapter_53);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper25221 = function() { return logError(function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1470, __wbg_adapter_56);
        return addHeapObject(ret);
    }, arguments) };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('black_hole_ray_marching_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
