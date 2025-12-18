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
        const rangeMinInteger: number = 10000;
        const rangeMaxInteger: number = 99_999_999;
        const randomInteger: number = this.randomGenerator.getRandomInteger(rangeMinInteger, rangeMaxInteger);
        const hexadecimalNumber: string = NumberUtils.toHex(randomInteger);
        const prefixLength: number = Utils.hexadecimalPrefix.length;
        const baseNameLength: number = (nameLength ?? 6) + prefixLength;
        const baseIdentifierName: string = hexadecimalNumber.slice(0, baseNameLength);
        const identifierName: string = `_${baseIdentifierName}`;

        if (!this.isValidIdentifierName(identifierName)) {
            return this.generateNext(nameLength);
        }

        this.preserveName(identifierName);

        return identifierName;
    }

    /**
     * Generate a name for global scope with optional prefix
     *
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForGlobalScope(nameLength?: number): string {
        const identifierName: string = this.generateNext(nameLength);

        return `${this.options.identifiersPrefix}${identifierName}`.replace('__', '_');
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
     * For labels, use the label itself as the original name
     *
     * @param {string} label
     * @param {number} nameLength
     * @returns {string}
     */
    public generateForLabel(label: string, nameLength?: number): string {
        return label;
    }
}
