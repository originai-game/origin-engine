// Capture intrinsics before project modules run. Validate before serializing:
// JSON.stringify's replacer runs after toJSON and silently drops some values.
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

    function encode(value, path, ancestors) {
        if (value === null) return 'null';
        switch (typeof value) {
            case 'string': return quote(value);
            case 'boolean': return value ? 'true' : 'false';
            case 'number':
                if (finite(value)) return quote(value);
                throw new Error(`${path}: non-finite generator output`);
            case 'object': break;
            default: throw new Error(`${path}: unsupported generator output (${typeof value})`);
        }
        for (let parent = ancestors; parent; parent = parent.parent) {
            if (parent.value === value) throw new Error(`${path}: cyclic generator output`);
        }
        const next = { value, parent: ancestors };
        // Existing generator contracts return numeric typed arrays. Preserve
        // their elements here; author IR will own binary buffers separately.
        if (isView(value)) {
            if (prototype(prototype(value)) !== typedPrototype) {
                throw new Error(`${path}: generator output needs a numeric typed array, not a view or subclass`);
            }
            const length = apply(typedLength, value, []);
            if (ownKeys(value).length !== length) {
                throw new Error(`${path}: generator typed arrays must have no extra properties`);
            }
            let text = '[';
            for (let i = 0; i < length; i++) {
                if (i) text += ',';
                text += encode(value[i], `${path}[${i}]`, next);
            }
            return text + ']';
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
            let text = '[';
            for (let i = 0; i < value.length; i++) {
                const field = descriptor(value, i);
                if (!field || !hasOwn(field, 'value') || !field.enumerable) {
                    throw new Error(`${path}[${i}]: generator output must be an enumerable data property`);
                }
                if (i) text += ',';
                text += encode(field.value, `${path}[${i}]`, next);
            }
            return text + ']';
        }
        // Sort object keys only, never array elements. Do not invoke getters,
        // toJSON or a user-installed iterator while inspecting source data.
        for (let i = 0; i < keys.length; i++) {
            if (typeof keys[i] !== 'string') throw new Error(`${path}: symbol keys are not source data`);
        }
        apply(sort, keys, []);
        let text = '{';
        for (let i = 0; i < keys.length; i++) {
            const key = keys[i];
            const field = descriptor(value, key);
            const at = `${path}[${quote(key)}]`;
            if (!hasOwn(field, 'value') || !field.enumerable) {
                throw new Error(`${at}: generator output must be an enumerable data property`);
            }
            if (i) text += ',';
            text += quote(key) + ':' + encode(field.value, at, next);
        }
        return text + '}';
    }
    return value => encode(value, '$', null);
})()
