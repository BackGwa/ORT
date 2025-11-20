/**
 * ORT Parser for TypeScript
 */

import { OrtValue, OrtValueType, createOrtValue } from './value';

export class OrtParseError extends Error {
    lineNum: number;
    line: string;

    constructor(lineNum: number, line: string, message: string) {
        super(`Line ${lineNum}: ${message}\n  ${line}`);
        this.lineNum = lineNum;
        this.line = line;
        this.name = 'OrtParseError';
    }
}

interface Field {
    name: string;
    nestedFields: Field[];
}

function createField(name: string, nestedFields: Field[] = []): Field {
    return { name, nestedFields };
}

function isNestedField(field: Field): boolean {
    return field.nestedFields.length > 0;
}

export function parseOrt(content: string): OrtValue {
    const lines = content.split('\n');
    let lineIdx = 0;
    const result: { [key: string]: any } = {};

    while (lineIdx < lines.length) {
        const line = lines[lineIdx].trim();

        // Skip empty lines and comments
        if (!line || line.startsWith('#')) {
            lineIdx++;
            continue;
        }

        // Parse header
        if (line.includes(':')) {
            const { key, fields, dataLines } = parseSection(lines, lineIdx);

            if (key !== null) {
                // keyName:fields: format
                const values = parseDataLines(lines, lineIdx + 1, fields, dataLines);
                result[key] = values.toNative();
                lineIdx += dataLines + 1;
            } else {
                // :fields: format (top-level)
                const values = parseDataLines(lines, lineIdx + 1, fields, dataLines);

                // If single object, return as object
                if (fields.length > 0 && dataLines === 1) {
                    if (values.isArray()) {
                        const arr = values.asArray();
                        if (arr && arr.length === 1) {
                            return arr[0];
                        }
                    }
                }
                return values;
            }
        } else {
            lineIdx++;
        }
    }

    return createOrtValue(result);
}

function parseSection(lines: string[], startIdx: number): { key: string | null; fields: Field[]; dataLines: number } {
    const line = lines[startIdx].trim();
    const lineNum = startIdx + 1;

    // Count data lines (non-empty, non-comment lines until next header or end)
    let dataLines = 0;
    for (let i = startIdx + 1; i < lines.length; i++) {
        const l = lines[i].trim();
        if (!l || l.startsWith('#')) {
            continue;
        }
        if (l.includes(':') && isHeader(l)) {
            break;
        }
        dataLines++;
    }

    // Parse header
    const { key, fieldsStr } = parseHeader(line, lineNum);
    const fields = parseFields(fieldsStr, line, lineNum);

    return { key, fields, dataLines };
}

function isHeader(line: string): boolean {
    const trimmed = line.trim();
    if (trimmed.startsWith(':')) {
        return true;
    }

    // Check if it's keyName:fields: format
    const parts = trimmed.split(':');
    if (parts.length >= 2 && parts[parts.length - 1] === '') {
        return true;
    }

    return false;
}

function parseHeader(line: string, lineNum: number): { key: string | null; fieldsStr: string } {
    if (line.startsWith(':')) {
        // :fields: format
        const content = line.substring(1).replace(/:$/, '');
        return { key: null, fieldsStr: content };
    } else {
        // keyName:fields: format
        const colonIndex = line.indexOf(':');
        if (colonIndex === -1) {
            throw new OrtParseError(lineNum, line, 'Invalid header format');
        }

        const key = line.substring(0, colonIndex).trim();
        const fields = line.substring(colonIndex + 1).replace(/:$/, '').trim();

        return { key, fieldsStr: fields };
    }
}

function parseFields(fieldsStr: string, line: string, lineNum: number): Field[] {
    if (!fieldsStr) {
        return [];
    }

    const result: Field[] = [];
    let current: string[] = [];
    let depth = 0;
    const chars = fieldsStr.split('');
    let i = 0;

    while (i < chars.length) {
        const ch = chars[i];

        if (ch === '(') {
            if (depth === 0) {
                // Start of nested fields
                const fieldName = current.join('').trim();
                current = [];
                i++;

                // Find matching closing paren
                const nestedStr: string[] = [];
                let nestedDepth = 1;

                while (i < chars.length && nestedDepth > 0) {
                    if (chars[i] === '(') {
                        nestedDepth++;
                    } else if (chars[i] === ')') {
                        nestedDepth--;
                    }

                    if (nestedDepth > 0) {
                        nestedStr.push(chars[i]);
                    }
                    i++;
                }

                const nestedFields = parseFields(nestedStr.join(''), line, lineNum);
                result.push(createField(fieldName, nestedFields));
                continue;
            } else {
                depth++;
                current.push(ch);
            }
        } else if (ch === ')') {
            depth--;
            if (depth < 0) {
                throw new OrtParseError(lineNum, line, 'Unmatched closing parenthesis');
            }
            current.push(ch);
        } else if (ch === ',') {
            if (depth === 0) {
                const field = current.join('').trim();
                if (field) {
                    result.push(createField(field));
                }
                current = [];
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }

        i++;
    }

    const field = current.join('').trim();
    if (field) {
        result.push(createField(field));
    }

    return result;
}

function parseDataLines(lines: string[], startIdx: number, fields: Field[], count: number): OrtValue {
    const result: any[] = [];
    let processed = 0;

    for (let i = startIdx; i < lines.length; i++) {
        if (processed >= count) {
            break;
        }

        const line = lines[i].trim();
        if (!line || line.startsWith('#')) {
            continue;
        }

        const lineNum = i + 1;

        // Special case: array value without fields
        if (fields.length === 0) {
            const value = parseValue(line, line, lineNum);
            return value;
        }

        // Parse data values
        const values = parseDataValues(line, lineNum);

        if (values.length !== fields.length) {
            throw new OrtParseError(
                lineNum,
                line,
                `Expected ${fields.length} values but got ${values.length}`
            );
        }

        const obj: { [key: string]: any } = {};
        for (let j = 0; j < fields.length; j++) {
            const field = fields[j];
            const valueStr = values[j];
            const value = parseFieldValue(field, valueStr, line, lineNum);
            obj[field.name] = value.toNative();
        }

        result.push(obj);
        processed++;
    }

    return createOrtValue(result);
}

function parseDataValues(line: string, lineNum: number): string[] {
    const values: string[] = [];
    let current: string[] = [];
    let escaped = false;
    let depth = 0;
    let bracketDepth = 0;

    for (const ch of line) {
        if (escaped) {
            current.push(ch);
            escaped = false;
            continue;
        }

        if (ch === '\\') {
            escaped = true;
            current.push('\\');
        } else if (ch === '(') {
            depth++;
            current.push(ch);
        } else if (ch === ')') {
            depth--;
            current.push(ch);
        } else if (ch === '[') {
            bracketDepth++;
            current.push(ch);
        } else if (ch === ']') {
            bracketDepth--;
            current.push(ch);
        } else if (ch === ',') {
            if (depth === 0 && bracketDepth === 0) {
                values.push(current.join(''));
                current = [];
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }
    }

    values.push(current.join(''));
    return values;
}

function parseFieldValue(field: Field, valueStr: string, line: string, lineNum: number): OrtValue {
    if (!isNestedField(field)) {
        return parseValue(valueStr, line, lineNum);
    }

    const trimmed = valueStr.trim();

    // Check for empty value
    if (!trimmed) {
        return createOrtValue(null);
    }

    // Check for empty object
    if (trimmed === '()') {
        return createOrtValue({});
    }

    // Handle array value dynamically (when field is defined as nested but value is array)
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        return parseValue(trimmed, line, lineNum);
    }

    // Parse nested object
    if (!trimmed.startsWith('(') || !trimmed.endsWith(')')) {
        // Fallback: parse as regular value if not in expected format
        return parseValue(trimmed, line, lineNum);
    }

    const inner = trimmed.substring(1, trimmed.length - 1);
    const values = parseDataValues(inner, lineNum);

    if (values.length !== field.nestedFields.length) {
        throw new OrtParseError(
            lineNum,
            line,
            `Expected ${field.nestedFields.length} nested values but got ${values.length}`
        );
    }

    const obj: { [key: string]: any } = {};
    for (let i = 0; i < field.nestedFields.length; i++) {
        const nestedField = field.nestedFields[i];
        const valueStr = values[i];
        const value = parseFieldValue(nestedField, valueStr, line, lineNum);
        obj[nestedField.name] = value.toNative();
    }

    return createOrtValue(obj);
}

function parseValue(s: string, line: string, lineNum: number): OrtValue {
    const trimmed = s.trim();

    // Empty value -> null
    if (!trimmed) {
        return createOrtValue(null);
    }

    // Empty array
    if (trimmed === '[]') {
        return createOrtValue([]);
    }

    // Empty object
    if (trimmed === '()') {
        return createOrtValue({});
    }

    // Array
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        return parseArray(trimmed.substring(1, trimmed.length - 1), line, lineNum);
    }

    // Inline object
    if (trimmed.startsWith('(') && trimmed.endsWith(')')) {
        return parseInlineObject(trimmed.substring(1, trimmed.length - 1), line, lineNum);
    }

    // Unescape string
    const unescaped = unescape(trimmed);

    // Try parse as number
    const num = Number(unescaped);
    if (!isNaN(num)) {
        return createOrtValue(num);
    }

    // Boolean
    if (unescaped === 'true') {
        return createOrtValue(true);
    }
    if (unescaped === 'false') {
        return createOrtValue(false);
    }

    // String
    return createOrtValue(unescaped);
}

function parseArray(s: string, line: string, lineNum: number): OrtValue {
    if (!s.trim()) {
        return createOrtValue([]);
    }

    const result: any[] = [];
    let current: string[] = [];
    let escaped = false;
    let depth = 0;
    let bracketDepth = 0;

    for (const ch of s) {
        if (escaped) {
            current.push(ch);
            escaped = false;
            continue;
        }

        if (ch === '\\') {
            escaped = true;
            current.push('\\');
        } else if (ch === '(') {
            depth++;
            current.push(ch);
        } else if (ch === ')') {
            depth--;
            current.push(ch);
        } else if (ch === '[') {
            bracketDepth++;
            current.push(ch);
        } else if (ch === ']') {
            bracketDepth--;
            current.push(ch);
        } else if (ch === ',') {
            if (depth === 0 && bracketDepth === 0) {
                const value = parseValue(current.join(''), line, lineNum);
                result.push(value.toNative());
                current = [];
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }
    }

    const currentStr = current.join('').trim();
    if (currentStr) {
        const value = parseValue(currentStr, line, lineNum);
        result.push(value.toNative());
    }

    return createOrtValue(result);
}

function parseInlineObject(s: string, line: string, lineNum: number): OrtValue {
    if (!s.trim()) {
        return createOrtValue({});
    }

    const obj: { [key: string]: any } = {};
    const pairs = splitPairs(s);

    for (const pair of pairs) {
        const colonIndex = pair.indexOf(':');
        if (colonIndex !== -1) {
            const key = pair.substring(0, colonIndex).trim();
            const valueStr = pair.substring(colonIndex + 1).trim();
            const value = parseValue(valueStr, line, lineNum);
            obj[key] = value.toNative();
        }
    }

    return createOrtValue(obj);
}

function splitPairs(s: string): string[] {
    const pairs: string[] = [];
    let current: string[] = [];
    let escaped = false;
    let depth = 0;
    let bracketDepth = 0;

    for (const ch of s) {
        if (escaped) {
            current.push(ch);
            escaped = false;
            continue;
        }

        if (ch === '\\') {
            escaped = true;
            current.push('\\');
        } else if (ch === '(') {
            depth++;
            current.push(ch);
        } else if (ch === ')') {
            depth--;
            current.push(ch);
        } else if (ch === '[') {
            bracketDepth++;
            current.push(ch);
        } else if (ch === ']') {
            bracketDepth--;
            current.push(ch);
        } else if (ch === ',') {
            if (depth === 0 && bracketDepth === 0) {
                pairs.push(current.join(''));
                current = [];
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }
    }

    const currentStr = current.join('').trim();
    if (currentStr) {
        pairs.push(currentStr);
    }

    return pairs;
}

function unescape(s: string): string {
    const result: string[] = [];
    let escaped = false;

    for (const ch of s) {
        if (escaped) {
            if (ch === 'n') {
                result.push('\n');
            } else if (ch === 't') {
                result.push('\t');
            } else if (ch === 'r') {
                result.push('\r');
            } else {
                result.push(ch);
            }
            escaped = false;
        } else if (ch === '\\') {
            escaped = true;
        } else {
            result.push(ch);
        }
    }

    return result.join('');
}
