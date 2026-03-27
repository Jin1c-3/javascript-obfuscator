import { inject, injectable } from 'inversify';
import { ServiceIdentifiers } from '../../container/ServiceIdentifiers';

import { TNodeWithLexicalScope } from '../../types/node/TNodeWithLexicalScope';

import { IOptions } from '../../interfaces/options/IOptions';
import { IRandomGenerator } from '../../interfaces/utils/IRandomGenerator';

import { AbstractIdentifierNamesGenerator } from './AbstractIdentifierNamesGenerator';
import { NumberUtils } from '../../utils/NumberUtils';
import { Utils } from '../../utils/Utils';

/**
 * This generator is used when identifierNamesGenerator is set to 'keep-original'.
 * In IdentifierReplacer and RenamePropertiesReplacer, the original names are used instead.
 * However, this generator still needs to provide valid names for internal use (e.g., string array).
 */
@injectable()
export class KeepOriginalIdentifierNamesGenerator extends AbstractIdentifierNamesGenerator {
    /**
     * @param {IRandomGenerator} randomGenerator
     * @param {IOptions} options
     */
    public constructor(
        @inject(ServiceIdentifiers.IRandomGenerator) randomGenerator: IRandomGenerator,
        @inject(ServiceIdentifiers.IOptions) options: IOptions
    ) {
        super(randomGenerator, options);
    }

    /**
     * Generate a simple hexadecimal name for internal use
     * This is used for things like string array names, not user identifiers
     *
     * @param {number} nameLength
     * @returns {string}
     */
    public generateNext(nameLength?: number): string {
        return this.generateNextName(nameLength, (name) => this.isValidIdentifierName(name));
    }

    /**
     * Generate a name for global scope with optional prefix
     *
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForGlobalScope(nameLength?: number): string {
        return this.generateForGlobalScopeInternal(nameLength, (name) => this.isValidIdentifierName(name));
    }

    /**
     * Generate a name for global scope with validation across all scopes
     *
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForGlobalScopeWithAllScopesValidation(nameLength?: number): string {
        return this.generateForGlobalScopeInternal(nameLength, (name) => this.isValidIdentifierNameInAllScopes(name));
    }

    /**
     * Generate a name for lexical scope
     *
     * @param {TNodeWithLexicalScope} lexicalScopeNode
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForLexicalScope(lexicalScopeNode: TNodeWithLexicalScope, nameLength?: number): string {
        return this.generateNext(nameLength);
    }

    /**
     * For internal labels (like control flow storage keys), generate a random name
     * This is not for user-defined labels, but for internal use
     *
     * @param {string} label
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForLabel(label: string, nameLength?: number): string {
        return this.generateNext(nameLength);
    }

    /**
     * @param {number} nameLength
     * @param {(name: string) => boolean} validationFn
     * @returns {string}
     */
    private generateNextName(nameLength: number | undefined, validationFn: (name: string) => boolean): string {
        const rangeMinInteger: number = 10000;
        const rangeMaxInteger: number = 99_999_999;
        const randomInteger: number = this.randomGenerator.getRandomInteger(rangeMinInteger, rangeMaxInteger);
        const hexadecimalNumber: string = NumberUtils.toHex(randomInteger);
        const prefixLength: number = Utils.hexadecimalPrefix.length;
        const baseNameLength: number = (nameLength ?? 6) + prefixLength;
        const baseIdentifierName: string = hexadecimalNumber.slice(0, baseNameLength);
        const identifierName: string = `_${baseIdentifierName}`;

        if (!validationFn(identifierName)) {
            return this.generateNextName(nameLength, validationFn);
        }

        this.preserveName(identifierName);

        return identifierName;
    }

    /**
     * @param {number} nameLength
     * @param {(name: string) => boolean} validationFn
     * @returns {string}
     */
    private generateForGlobalScopeInternal(nameLength: number | undefined, validationFn: (name: string) => boolean): string {
        const identifierName: string = this.generateNextName(nameLength, validationFn);

        return `${this.options.identifiersPrefix}${identifierName}`.replace('__', '_');
    }
}
