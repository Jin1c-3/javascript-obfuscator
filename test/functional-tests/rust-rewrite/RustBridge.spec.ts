import { assert } from 'chai';

import { JavaScriptObfuscator } from '../../../src/JavaScriptObfuscatorFacade';

describe('Rust rewrite compatibility boundary', () => {
    describe('Variant #1: public facade remains available', () => {
        it('should obfuscate code through the existing facade', () => {
            const result = JavaScriptObfuscator.obfuscate('const value = 1; console.log(value);', {
                compact: true,
                stringArray: false,
                renameGlobals: false
            });

            assert.isString(result.getObfuscatedCode());
            assert.include(result.getObfuscatedCode(), 'console');
        });
    });

    describe('Variant #2: multiple source facade remains available', () => {
        it('should obfuscate multiple source files', () => {
            const results = JavaScriptObfuscator.obfuscateMultiple(
                {
                    'first.js': 'const first = 1;',
                    'second.js': 'const second = 2;'
                },
                {
                    compact: true,
                    stringArray: false,
                    renameGlobals: false
                }
            );

            assert.hasAllKeys(results, ['first.js', 'second.js']);
            assert.isString(results['first.js'].getObfuscatedCode());
            assert.isString(results['second.js'].getObfuscatedCode());
        });
    });

    describe('Variant #3: removed async API is not exposed', () => {
        it('should not expose the removed async facade method', () => {
            const removedMethodName: string = ['obfuscate', 'Pro'].join('');

            assert.isUndefined((JavaScriptObfuscator as unknown as Record<string, unknown>)[removedMethodName]);
        });
    });

    describe('Variant #4: Rust bridge scaffold', () => {
        it('should expose a disabled bridge until Rust parity is ready', () => {
            const { RustObfuscatorBridge } = require('../../../src/rust/RustObfuscatorBridge');

            assert.isFalse(RustObfuscatorBridge.isAvailable());
            assert.isNull(RustObfuscatorBridge.obfuscate('const value = 1;', {}));
        });
    });

    describe('Variant #5: package facade stays on TypeScript engine for this slice', () => {
        it('should keep TypeScript transformer behavior while Rust parser parity is incomplete', () => {
            const result = JavaScriptObfuscator.obfuscate('const value = "hello"; console.log(value);', {
                compact: true,
                stringArray: false,
                renameGlobals: false
            });

            assert.include(result.getObfuscatedCode(), 'console');
        });
    });
});
