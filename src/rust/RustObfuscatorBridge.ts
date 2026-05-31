import { TInputOptions } from '../types/options/TInputOptions';

interface IRustObfuscationPayload {
    readonly code: string;
    readonly sourceMap: string;
    readonly identifierNamesCache?: unknown;
}

export class RustObfuscatorBridge {
    public static isAvailable(): boolean {
        return false;
    }

    public static obfuscate(sourceCode: string, inputOptions: TInputOptions): IRustObfuscationPayload | null {
        return sourceCode && inputOptions ? null : null;
    }

    public static getOptionsByPreset(optionsPreset: string): TInputOptions | null {
        return optionsPreset ? null : null;
    }
}
