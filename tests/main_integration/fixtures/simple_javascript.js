// 这是一个简单的JavaScript文件，用于测试翻译功能
// 包含中文注释

/**
 * 计算两个数的和
 * @param {number} a - 第一个数字
 * @param {number} b - 第二个数字
 * @returns {number} 两个数的和
 */
function add(a, b) {
    return a + b;
}

/**
 * 计算两个数的差
 * @param {number} a - 被减数
 * @param {number} b - 减数
 * @returns {number} 两个数的差
 */
function subtract(a, b) {
    return a - b;
}

// 简单的类
class Calculator {
    /**
     * 创建计算器实例
     * @param {string} name - 计算器名称
     */
    constructor(name) {
        this.name = name;
    }
    
    /**
     * 获取计算器名称
     * @returns {string} 计算器名称
     */
    getName() {
        return this.name;
    }
}

// 主函数
function main() {
    console.log("测试翻译功能");
    const result = add(10, 20);
    console.log(`结果: ${result}`);
}

main();
