import * as fs from 'fs';
import { OrtValue, OrtValueType, createOrtValue } from './value';
import { parseOrt, OrtParseError } from './parser';
import { generateOrt } from './generator';

export { OrtValue, OrtValueType, createOrtValue, OrtParseError };

export function parse(content: string): OrtValue {
    return parseOrt(content);
}

export function generate(value: OrtValueType): string {
    const ortValue = createOrtValue(value);
    return generateOrt(ortValue);
}

export function load(filePath: string): OrtValue {
    const content = fs.readFileSync(filePath, 'utf-8');
    return parseOrt(content);
}

export function dump(value: OrtValueType, filePath: string): void {
    const ortValue = createOrtValue(value);
    const ortString = generateOrt(ortValue);
    fs.writeFileSync(filePath, ortString, 'utf-8');
}

export default {
    parse,
    generate,
    load,
    dump,
    OrtValue,
    createOrtValue,
    OrtParseError
};
