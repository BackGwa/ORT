/**
 * ORT Generator for TypeScript
 */

import { OrtValue, OrtValueType } from './value';

export function generateOrt(value: OrtValue): string {
    if (value.isObject()) {
        const obj = value.asObject();
        if (obj) {
            const objNative = value.toNative() as { [key: string]: any };

            // Check if this is a multi-key object
            if (Object.keys(objNative).length > 1 || Object.keys(objNative).length === 0) {
                return generateMultiObject(objNative);
            } else if (Object.keys(objNative).length === 1) {
                // Single key - might be a named array
                const [key, val] = Object.entries(objNative)[0];
                const valOrt = new OrtValue(val);
                if (valOrt.isArray()) {
                    const arr = valOrt.asArray();
                    if (!arr || arr.length === 0) {
                        return `${key}:\n[]`;
                    } else {
                        const arrList = arr.map(v => v.toNative());
                        if (isUniformObjectArray(arrList)) {
                            return generateObjectArray(key, arrList);
                        } else {
                            return generateSimpleArray(key, arrList);
                        }
                    }
                } else {
                    // Single key with non-array value
                    return `${key}:\n${generateValue(val, false)}`;
                }
            }
        }
        return '';
    } else if (value.isArray()) {
        const arr = value.asArray();
        if (arr) {
            const arrList = arr.map(v => v.toNative());
            // Top-level array
            if (isUniformObjectArray(arrList)) {
                return generateTopLevelObjectArray(arrList);
            } else {
                return `:${generateArrayContent(arrList, false)}`;
            }
        }
        return ':[]';
    } else {
        return generateValue(value.toNative(), false);
    }
}

function generateMultiObject(obj: { [key: string]: any }): string {
    const result: string[] = [];
    const entries = Object.entries(obj);

    for (let i = 0; i < entries.length; i++) {
        const [key, val] = entries[i];
        const valOrt = new OrtValue(val);
        if (valOrt.isArray()) {
            const arr = valOrt.asArray();
            if (!arr || arr.length === 0) {
                result.push(`${key}:\n[]`);
            } else {
                const arrList = arr.map(v => v.toNative());
                if (isUniformObjectArray(arrList)) {
                    result.push(generateObjectArray(key, arrList).trimEnd());
                } else {
                    result.push(generateSimpleArray(key, arrList).trimEnd());
                }
            }
        } else {
            result.push(`${key}:\n${generateValue(val, false)}`);
        }

        if (i < entries.length - 1) {
            result.push('\n\n');
        } else {
            result.push('\n');
        }
    }

    return result.join('');
}

function isUniformObjectArray(arr: any[]): boolean {
    if (arr.length === 0) {
        return false;
    }

    // Check if all elements are objects with the same keys
    if (typeof arr[0] !== 'object' || arr[0] === null || Array.isArray(arr[0])) {
        return false;
    }

    const firstKeys = Object.keys(arr[0]).sort();

    for (let i = 1; i < arr.length; i++) {
        const item = arr[i];
        if (typeof item !== 'object' || item === null || Array.isArray(item)) {
            return false;
        }
        const keys = Object.keys(item).sort();
        if (JSON.stringify(keys) !== JSON.stringify(firstKeys)) {
            return false;
        }
    }

    return true;
}

function generateObjectArray(key: string, arr: { [key: string]: any }[]): string {
    if (arr.length === 0) {
        return `${key}:\n[]`;
    }

    const first = arr[0];
    const keys = Object.keys(first);
    const header = generateHeader(keys, first);

    const result: string[] = [`${key}:${header}`];

    for (const item of arr) {
        const values = keys.map(k => generateObjectFieldValue(item[k], keys, k, item));
        result.push(values.join(','));
    }

    return result.join('\n');
}

function generateTopLevelObjectArray(arr: { [key: string]: any }[]): string {
    if (arr.length === 0) {
        return ':[]';
    }

    const first = arr[0];
    const keys = Object.keys(first);
    const header = generateHeader(keys, first);

    const result: string[] = [`:${header}`];

    for (const item of arr) {
        const values = keys.map(k => generateObjectFieldValue(item[k], keys, k, item));
        result.push(values.join(','));
    }

    return result.join('\n');
}

function generateHeader(keys: string[], firstObj: { [key: string]: any }): string {
    const headerParts: string[] = [];

    for (const k of keys) {
        const value = firstObj[k];
        if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
            // Generate nested field
            const nestedKeys = Object.keys(value);
            const nestedHeader = generateHeaderFields(nestedKeys, value);
            headerParts.push(`${k}(${nestedHeader})`);
        } else {
            headerParts.push(k);
        }
    }

    return headerParts.join(',') + ':';
}

function generateHeaderFields(keys: string[], obj: { [key: string]: any }): string {
    const headerParts: string[] = [];

    for (const k of keys) {
        const value = obj[k];
        if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
            // Recursively generate nested field
            const nestedKeys = Object.keys(value);
            const nestedHeader = generateHeaderFields(nestedKeys, value);
            headerParts.push(`${k}(${nestedHeader})`);
        } else {
            headerParts.push(k);
        }
    }

    return headerParts.join(',');
}

function generateObjectFieldValue(
    value: any,
    keys: string[],
    currentKey: string,
    parent: { [key: string]: any }
): string {
    if (value === null || value === undefined) {
        return '';
    } else if (typeof value === 'object' && !Array.isArray(value)) {
        if (Object.keys(value).length === 0) {
            return '()';
        } else {
            // Nested object - output values only (keys are in header)
            const nestedKeys = Object.keys(value);
            const values = nestedKeys.map(k => generateObjectFieldValue(value[k], nestedKeys, k, value));
            return `(${values.join(',')})`;
        }
    } else if (Array.isArray(value)) {
        if (value.length === 0) {
            return '[]';
        } else {
            return `[${generateArrayContent(value, true)}]`;
        }
    } else {
        return generateValue(value, true);
    }
}

function generateSimpleArray(key: string, arr: any[]): string {
    return `${key}:\n${generateArrayContent(arr, false)}`;
}

function generateArrayContent(arr: any[], inline: boolean): string {
    if (arr.length === 0) {
        return '[]';
    }

    const values = arr.map(v => generateValue(v, inline));

    if (inline) {
        return values.join(',');
    } else {
        return `[${values.join(',')}]`;
    }
}

function generateValue(value: any, inline: boolean): string {
    if (value === null || value === undefined) {
        return '';
    } else if (typeof value === 'boolean') {
        return value ? 'true' : 'false';
    } else if (typeof value === 'number') {
        // Format number nicely (remove .0 for whole numbers)
        if (Number.isInteger(value)) {
            return String(value);
        }
        return String(value);
    } else if (typeof value === 'string') {
        return escape(value);
    } else if (Array.isArray(value)) {
        if (value.length === 0) {
            return '[]';
        } else {
            return `[${generateArrayContent(value, true)}]`;
        }
    } else if (typeof value === 'object') {
        if (Object.keys(value).length === 0) {
            return '()';
        } else {
            return generateInlineObject(value);
        }
    } else {
        return String(value);
    }
}

function generateInlineObject(obj: { [key: string]: any }): string {
    const pairs = Object.entries(obj).map(([k, v]) => `${k}:${generateValue(v, true)}`);
    return `(${pairs.join(',')})`;
}

function escape(s: string): string {
    const result: string[] = [];

    for (const ch of s) {
        if (ch === '(') {
            result.push('\\(');
        } else if (ch === ')') {
            result.push('\\)');
        } else if (ch === '[') {
            result.push('\\[');
        } else if (ch === ']') {
            result.push('\\]');
        } else if (ch === ',') {
            result.push('\\,');
        } else if (ch === '\\') {
            result.push('\\\\');
        } else if (ch === '\n') {
            result.push('\\n');
        } else if (ch === '\t') {
            result.push('\\t');
        } else if (ch === '\r') {
            result.push('\\r');
        } else {
            result.push(ch);
        }
    }

    return result.join('');
}
