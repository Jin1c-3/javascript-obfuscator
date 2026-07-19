import { inject, injectable, injectFromBase } from 'inversify';
import { ServiceIdentifiers } from '../../container/ServiceIdentifiers';

import * as ESTree from 'estree';

import { TIdentifierNamesGeneratorFactory } from '../../types/container/generators/TIdentifierNamesGeneratorFactory';
import { TStatement } from '../../types/node/TStatement';

import { StringSeparator } from '../../enums/StringSeparator';

import { ICustomCodeHelperFormatter } from '../../interfaces/custom-code-helpers/ICustomCodeHelperFormatter';
import { IOptions } from '../../interfaces/options/IOptions';
import { IRandomGenerator } from '../../interfaces/utils/IRandomGenerator';

import { initializable } from '../../decorators/Initializable';

import { AbstractCustomNode } from '../AbstractCustomNode';
import { NodeFactory } from '../../node/NodeFactory';
import { NodeGuards } from '../../node/NodeGuards';
import { NodeUtils } from '../../node/NodeUtils';

@injectFromBase()
@injectable()
export class BlockStatementControlFlowFlatteningNode extends AbstractCustomNode {
    /**
     * @type {ESTree.Statement[]}
     */
    @initializable()
    private blockStatementBody!: ESTree.Statement[];

    /**
     * @type {number[]}
     */
    @initializable()
    private originalKeysIndexesInShuffledArray!: number[];

    /**
     * @type {number[]}
     */
    @initializable()
    private shuffledKeys!: number[];

    /**
     * @param {TIdentifierNamesGeneratorFactory} identifierNamesGeneratorFactory
     * @param {ICustomCodeHelperFormatter} customCodeHelperFormatter
     * @param {IRandomGenerator} randomGenerator
     * @param {IOptions} options
     */
    public constructor(
        @inject(ServiceIdentifiers.Factory__IIdentifierNamesGenerator)
        identifierNamesGeneratorFactory: TIdentifierNamesGeneratorFactory,
        @inject(ServiceIdentifiers.ICustomCodeHelperFormatter) customCodeHelperFormatter: ICustomCodeHelperFormatter,
        @inject(ServiceIdentifiers.IRandomGenerator) randomGenerator: IRandomGenerator,
        @inject(ServiceIdentifiers.IOptions) options: IOptions
    ) {
        super(identifierNamesGeneratorFactory, customCodeHelperFormatter, randomGenerator, options);
    }

    /**
     * @param {Statement[]} blockStatementBody
     * @param {number[]} shuffledKeys
     * @param {number[]} originalKeysIndexesInShuffledArray
     */
    public initialize(
        blockStatementBody: ESTree.Statement[],
        shuffledKeys: number[],
        originalKeysIndexesInShuffledArray: number[]
    ): void {
        this.blockStatementBody = blockStatementBody;
        this.shuffledKeys = shuffledKeys;
        this.originalKeysIndexesInShuffledArray = originalKeysIndexesInShuffledArray;
    }

    /**
     * @returns {TStatement[]}
     */
    protected getNodeStructure(): TStatement[] {
        const controllerIdentifierName: string = this.randomGenerator.getRandomString(6);
        const indexIdentifierName: string = this.randomGenerator.getRandomString(6);

        const structure: ESTree.BlockStatement = NodeFactory.blockStatementNode([
            NodeFactory.variableDeclarationNode(
                [
                    NodeFactory.variableDeclaratorNode(
                        NodeFactory.identifierNode(controllerIdentifierName),
                        // The dispatch-order table is emitted reversed and restored with a trailing
                        // `.reverse()`: `"2|1|3|0".split("|").reverse()` instead of `"0|3|1|2".split("|")`.
                        // Semantically identical (same array at runtime), but the order is no longer a bare
                        // `<stringLiteral>.split("|")` — the exact shape structural CFF unflatteners key on
                        // (webcrack's control-flow-switch demands a `stringLiteral.split("|")` initializer;
                        // talon's array-driven unflatten reads only a single `.split()`/array literal). The
                        // order stays a pure constant expression, so a normalizer can fold it back to the
                        // canonical form and hand the dispatcher to those unflatteners.
                        NodeFactory.callExpressionNode(
                            NodeFactory.memberExpressionNode(
                                NodeFactory.callExpressionNode(
                                    NodeFactory.memberExpressionNode(
                                        NodeFactory.literalNode(
                                            [...this.originalKeysIndexesInShuffledArray]
                                                .reverse()
                                                .join(StringSeparator.VerticalLine)
                                        ),
                                        NodeFactory.identifierNode('split')
                                    ),
                                    [NodeFactory.literalNode(StringSeparator.VerticalLine)]
                                ),
                                NodeFactory.identifierNode('reverse')
                            ),
                            []
                        )
                    )
                ],
                'const'
            ),
            NodeFactory.variableDeclarationNode(
                [
                    NodeFactory.variableDeclaratorNode(
                        NodeFactory.identifierNode(indexIdentifierName),
                        NodeFactory.literalNode(0)
                    )
                ],
                'let'
            ),
            NodeFactory.whileStatementNode(
                NodeFactory.literalNode(true),
                NodeFactory.blockStatementNode([
                    NodeFactory.switchStatementNode(
                        NodeFactory.memberExpressionNode(
                            NodeFactory.identifierNode(controllerIdentifierName),
                            NodeFactory.updateExpressionNode('++', NodeFactory.identifierNode(indexIdentifierName)),
                            true
                        ),
                        this.shuffledKeys.map((key: number, index: number) => {
                            const statement: ESTree.Statement = this.blockStatementBody[key];
                            const consequent: ESTree.Statement[] = [statement];

                            /**
                             * We shouldn't add continue statement after return statement
                             * to prevent `unreachable code after return statement` warnings
                             */
                            if (!NodeGuards.isReturnStatementNode(statement)) {
                                consequent.push(NodeFactory.continueStatement());
                            }

                            return NodeFactory.switchCaseNode(NodeFactory.literalNode(String(index)), consequent);
                        })
                    ),
                    NodeFactory.breakStatement()
                ])
            )
        ]);

        NodeUtils.parentizeAst(structure);

        return [structure];
    }
}
