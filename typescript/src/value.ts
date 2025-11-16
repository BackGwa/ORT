/**
 * ORT Value type for TypeScript
 */

export type OrtValueType = null | boolean | number | string | OrtValueType[] | { [key: string]: OrtValueType };

export class OrtValue {
    private _value: OrtValueType;

    constructor(value: OrtValueType) {
        this._value = this.normalize(value);
    }

    private normalize(value: any): OrtValueType {
        if (value === null || value === undefined) {
            return null;
        } else if (typeof value === 'boolean') {
            return value;
        } else if (typeof value === 'number') {
            return value;
        } else if (typeof value === 'string') {
            return value;
        } else if (Array.isArray(value)) {
            return value.map(v => new OrtValue(v)._value);
        } else if (typeof value === 'object') {
            if (value instanceof OrtValue) {
                return value._value;
            }
            const result: { [key: string]: OrtValueType } = {};
            for (const [k, v] of Object.entries(value as any)) {
                result[k] = new OrtValue(v as any)._value;
            }
            return result;
        } else {
            throw new TypeError(`Unsupported type for OrtValue: ${typeof value}`);
        }
    }

    isNull(): boolean {
        return this._value === null;
    }

    isBool(): boolean {
        return typeof this._value === 'boolean';
    }

    isNumber(): boolean {
        return typeof this._value === 'number';
    }

    isString(): boolean {
        return typeof this._value === 'string';
    }

    isArray(): boolean {
        return Array.isArray(this._value);
    }

    isObject(): boolean {
        return this._value !== null && typeof this._value === 'object' && !Array.isArray(this._value);
    }

    asBool(): boolean | null {
        return typeof this._value === 'boolean' ? this._value : null;
    }

    asNumber(): number | null {
        return typeof this._value === 'number' ? this._value : null;
    }

    asString(): string | null {
        return typeof this._value === 'string' ? this._value : null;
    }

    asArray(): OrtValue[] | null {
        if (Array.isArray(this._value)) {
            return this._value.map(v => new OrtValue(v));
        }
        return null;
    }

    asObject(): { [key: string]: OrtValue } | null {
        if (this.isObject()) {
            const result: { [key: string]: OrtValue } = {};
            for (const [k, v] of Object.entries(this._value as { [key: string]: OrtValueType })) {
                result[k] = new OrtValue(v);
            }
            return result;
        }
        return null;
    }

    get(key: string | number): OrtValue {
        if (typeof key === 'string') {
            if (!this.isObject()) {
                throw new TypeError(`Cannot index non-object with string key: ${key}`);
            }
            const obj = this._value as { [key: string]: OrtValueType };
            if (!(key in obj)) {
                throw new Error(`Key not found: ${key}`);
            }
            return new OrtValue(obj[key]);
        } else if (typeof key === 'number') {
            if (!Array.isArray(this._value)) {
                throw new TypeError(`Cannot index non-array with number: ${key}`);
            }
            if (key < 0 || key >= this._value.length) {
                throw new Error(`Index out of bounds: ${key}`);
            }
            return new OrtValue(this._value[key]);
        } else {
            throw new TypeError(`Key must be string or number, got ${typeof key}`);
        }
    }

    set(key: string | number, value: OrtValueType): void {
        if (typeof key === 'string') {
            if (!this.isObject()) {
                throw new TypeError(`Cannot set key on non-object: ${key}`);
            }
            const obj = this._value as { [key: string]: OrtValueType };
            obj[key] = new OrtValue(value)._value;
        } else if (typeof key === 'number') {
            if (!Array.isArray(this._value)) {
                throw new TypeError(`Cannot set index on non-array: ${key}`);
            }
            if (key < 0 || key >= this._value.length) {
                throw new Error(`Index out of bounds: ${key}`);
            }
            this._value[key] = new OrtValue(value)._value;
        } else {
            throw new TypeError(`Key must be string or number, got ${typeof key}`);
        }
    }

    getOrDefault(key: string, defaultValue: OrtValueType = null): OrtValue {
        if (!this.isObject()) {
            return new OrtValue(defaultValue);
        }
        const obj = this._value as { [key: string]: OrtValueType };
        return new OrtValue(obj[key] ?? defaultValue);
    }

    toNative(): any {
        if (this._value === null) {
            return null;
        } else if (typeof this._value === 'boolean' || typeof this._value === 'number' || typeof this._value === 'string') {
            return this._value;
        } else if (Array.isArray(this._value)) {
            return this._value.map(v => new OrtValue(v).toNative());
        } else if (typeof this._value === 'object') {
            const result: { [key: string]: any } = {};
            for (const [k, v] of Object.entries(this._value)) {
                result[k] = new OrtValue(v).toNative();
            }
            return result;
        }
        return this._value;
    }

    toString(): string {
        return `OrtValue(${JSON.stringify(this._value)})`;
    }

    equals(other: OrtValue | OrtValueType): boolean {
        if (other instanceof OrtValue) {
            return JSON.stringify(this._value) === JSON.stringify(other._value);
        }
        return JSON.stringify(this._value) === JSON.stringify(other);
    }

    get length(): number {
        if (Array.isArray(this._value)) {
            return this._value.length;
        } else if (this.isObject()) {
            return Object.keys(this._value as object).length;
        }
        throw new TypeError(`Object of type ${typeof this._value} has no length`);
    }

    // Support bracket notation: value["key"] or value[0]
    [key: string]: any;
}

// Add Proxy support for bracket notation
export function createOrtValue(value: OrtValueType): OrtValue & { [key: string]: any; [key: number]: any } {
    const ortValue = new OrtValue(value);

    return new Proxy(ortValue, {
        get(target, prop) {
            if (prop in target) {
                return (target as any)[prop];
            }
            if (typeof prop === 'string') {
                if (!isNaN(Number(prop))) {
                    return target.get(Number(prop));
                }
                return target.get(prop);
            }
            return undefined;
        },
        set(target, prop, value) {
            if (typeof prop === 'string') {
                if (!isNaN(Number(prop))) {
                    target.set(Number(prop), value);
                } else {
                    target.set(prop, value);
                }
                return true;
            }
            return false;
        }
    }) as OrtValue & { [key: string]: any; [key: number]: any };
}
