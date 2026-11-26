// Capture intrinsics before project modules run. Write the private author IR
// directly, never JSON numeric strings. Layout and limits: generator/value.rs.
(() => {
    const ownKeys = Reflect.ownKeys;
    const descriptor = Object.getOwnPropertyDescriptor;
    const hasOwn = Object.hasOwn;
    const prototype = Object.getPrototypeOf;
    const objectPrototype = Object.prototype;
    const arrayPrototype = Array.prototype;
    const isArray = Array.isArray;
    const isView = ArrayBuffer.isView;
    const finite = Number.isFinite;
    const quote = JSON.stringify;
    const apply = Reflect.apply;
    const sort = Array.prototype.sort;
    const typedPrototype = prototype(Uint8Array.prototype);
    const typedLength = descriptor(typedPrototype, 'length').get;
    const typedBuffer = descriptor(typedPrototype, 'buffer').get;
    const typedOffset = descriptor(typedPrototype, 'byteOffset').get;
    const typedBytes = descriptor(typedPrototype, 'byteLength').get;
    const typedName = descriptor(typedPrototype, Symbol.toStringTag).get;
    const Bytes = Uint8Array;
    const View = DataView;
    const setBytes = Bytes.prototype.set;
    const charCodeAt = String.prototype.charCodeAt;
    const set16 = View.prototype.setUint16;
    const set32 = View.prototype.setUint32;
    const set64 = View.prototype.setFloat64;
    const littleEndian = new Bytes(new Uint16Array([1]).buffer)[0] === 1;
    const types = [
        [Int8Array.prototype, 16, 1, 'Int8Array'], [Uint8Array.prototype, 17, 1, 'Uint8Array'],
        [Uint8ClampedArray.prototype, 18, 1, 'Uint8ClampedArray'], [Int16Array.prototype, 19, 2, 'Int16Array'],
        [Uint16Array.prototype, 20, 2, 'Uint16Array'], [Int32Array.prototype, 21, 4, 'Int32Array'],
        [Uint32Array.prototype, 22, 4, 'Uint32Array'], [Float32Array.prototype, 23, 4, 'Float32Array'],
        [Float64Array.prototype, 24, 8, 'Float64Array'],
    ];
    const maxBytes = 64 * 1024 * 1024;
    let bytes, view, used, nodes;

    function reserve(size) {
        const end = used + size;
        if (end > maxBytes) throw new Error('author output exceeds 64 MiB; split the content entry');
        if (end > apply(typedLength, bytes, [])) {
            let capacity = apply(typedLength, bytes, []);
            while (capacity < end) capacity *= 2;
            const next = new Bytes(capacity);
            apply(setBytes, next, [bytes]);
            bytes = next;
            view = new View(apply(typedBuffer, bytes, []));
        }
        const start = used;
        used = end;
        return start;
    }
    function tag(value) { const at = reserve(1); bytes[at] = value; }
    function count(value) { const at = reserve(4); apply(set32, view, [at, value, true]); }
    function string(value) {
        count(value.length);
        for (let i = 0; i < value.length; i++) {
            const at = reserve(2);
            apply(set16, view, [at, apply(charCodeAt, value, [i]), true]);
        }
    }

    function encode(value, path, ancestors) {
        if (++nodes > 1000000) throw new Error(`${path}: author output exceeds one million nodes; use typed arrays or split the entry`);
        if (value === null) { tag(0); return; }
        switch (typeof value) {
            case 'string': tag(4); string(value); return;
            case 'boolean': tag(value ? 2 : 1); return;
            case 'number':
                if (finite(value)) {
                    tag(3);
                    const at = reserve(8);
                    apply(set64, view, [at, value === 0 ? 0 : value, true]);
                    return;
                }
                throw new Error(`${path}: non-finite generator output`);
            case 'object': break;
            default: throw new Error(`${path}: unsupported generator output (${typeof value})`);
        }
        for (let parent = ancestors; parent; parent = parent.parent) {
            if (parent.value === value) throw new Error(`${path}: cyclic generator output`);
        }
        const depth = ancestors ? ancestors.depth + 1 : 1;
        if (depth > 128) throw new Error(`${path}: author output nesting exceeds 128`);
        const next = { value, parent: ancestors, depth };
        if (isView(value)) {
            const proto = prototype(value);
            const name = apply(typedName, value, []);
            let type;
            for (let i = 0; i < types.length; i++) {
                if (proto === types[i][0] && name === types[i][3]) { type = types[i]; break; }
            }
            if (!type) throw new Error(`${path}: generator output needs a numeric typed array, not a view or subclass`);
            const length = apply(typedLength, value, []);
            if (ownKeys(value).length !== length) {
                throw new Error(`${path}: generator typed arrays must have no extra properties`);
            }
            if (type[1] >= 23) {
                for (let i = 0; i < length; i++) {
                    if (!finite(value[i])) throw new Error(`${path}[${i}]: non-finite generator output`);
                }
            }
            tag(type[1]); count(length);
            const byteLength = apply(typedBytes, value, []);
            const raw = new Bytes(apply(typedBuffer, value, []), apply(typedOffset, value, []), byteLength);
            const start = reserve(byteLength);
            if (littleEndian || type[2] === 1) apply(setBytes, bytes, [raw, start]);
            else {
                for (let i = 0; i < length; i++) {
                    for (let b = 0; b < type[2]; b++) bytes[start + i * type[2] + b] = raw[i * type[2] + type[2] - b - 1];
                }
            }
            return;
        }
        const array = isArray(value);
        const proto = prototype(value);
        if (array ? proto !== arrayPrototype : proto !== null && proto !== objectPrototype) {
            throw new Error(`${path}: generator output must contain plain objects, not class instances`);
        }
        const keys = ownKeys(value);
        if (array) {
            if (keys.length !== value.length + 1) {
                throw new Error(`${path}: generator arrays must be dense and have no extra properties`);
            }
            tag(5); count(value.length);
            for (let i = 0; i < value.length; i++) {
                const field = descriptor(value, i);
                if (!field || !hasOwn(field, 'value') || !field.enumerable) {
                    throw new Error(`${path}[${i}]: generator output must be an enumerable data property`);
                }
                encode(field.value, `${path}[${i}]`, next);
            }
            return;
        }
        // Sort object keys only, never array elements. Do not invoke getters,
        // toJSON or a user-installed iterator while inspecting source data.
        for (let i = 0; i < keys.length; i++) {
            if (typeof keys[i] !== 'string') throw new Error(`${path}: symbol keys are not source data`);
        }
        apply(sort, keys, []);
        tag(6); count(keys.length);
        for (let i = 0; i < keys.length; i++) {
            const key = keys[i];
            const field = descriptor(value, key);
            const at = `${path}[${quote(key)}]`;
            if (!hasOwn(field, 'value') || !field.enumerable) {
                throw new Error(`${at}: generator output must be an enumerable data property`);
            }
            string(key);
            encode(field.value, at, next);
        }
    }
    return value => {
        bytes = new Bytes(256);
        view = new View(apply(typedBuffer, bytes, []));
        used = 0; nodes = 0;
        tag(79); tag(71); tag(65); tag(86); tag(1); // OGAV, private layout 1
        encode(value, '$', null);
        // TypedArray#subarray consults user-editable constructor/species hooks.
        // A direct view over our private buffer cannot invoke those hooks.
        return new Bytes(apply(typedBuffer, bytes, []), 0, used);
    };
})()
