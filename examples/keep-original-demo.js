// 示例：使用 keep-original 模式
// 这个文件展示了如何使用新的 keep-original identifierNamesGenerator 选项

const JavaScriptObfuscator = require('../dist/index');

// 测试代码
const sourceCode = `
function calculateSum(firstNumber, secondNumber) {
    const result = firstNumber + secondNumber;
    console.log('Result:', result);
    return result;
}

const myVariable = 'Hello';
const myObject = {
    propertyName: 'value',
    methodName: function() {
        return this.propertyName;
    }
};

calculateSum(10, 20);
`;

console.log('=== 原始代码 ===');
console.log(sourceCode);

console.log('\n=== 使用 keep-original 模式混淆 ===');
const obfuscatedWithKeepOriginal = JavaScriptObfuscator.obfuscate(sourceCode, {
    compact: false,
    identifierNamesGenerator: 'keep-original'
});
console.log(obfuscatedWithKeepOriginal.getObfuscatedCode());

console.log('\n=== 使用 hexadecimal 模式混淆（对比） ===');
const obfuscatedWithHex = JavaScriptObfuscator.obfuscate(sourceCode, {
    compact: false,
    identifierNamesGenerator: 'hexadecimal'
});
console.log(obfuscatedWithHex.getObfuscatedCode());
