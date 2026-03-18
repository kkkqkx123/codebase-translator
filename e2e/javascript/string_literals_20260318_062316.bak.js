/**
 * JavaScript String Literals Fixture
 * 
 * Tests extraction of string literals from various JavaScript code patterns.
 */

// Error message strings
function errorExamples() {
    throw new Error("系统遇到致命错误");
    throw new Error("Fatal system error occurred");
    
    throw new TypeError("参数类型不正确");
    throw new RangeError("Value out of range");
    
    console.error("操作失败：", "无法保存数据");
    console.error("Operation failed:", "Cannot save data");
}

// Console/log output strings
function logExamples() {
    console.log("应用程序启动成功");
    console.log("Application started successfully");
    
    console.warn("警告：内存使用率高");
    console.warn("Warning: High memory usage");
    
    console.info("用户已登录");
    console.info("User logged in");
    
    console.debug("调试信息：", "variable =", 42);
    console.debug("Debug info:", "variable =", 42);
}

// Template literals (backticks)
function templateLiteralExamples() {
    const name = "王五";
    const age = 28;
    
    const msg1 = `用户 ${name} 的年龄是 ${age}`;
    const msg2 = `User ${name} is ${age} years old`;
    const msg3 = `Processing ${1} of ${10}`;
    
    // Mixed language
    const msg4 = `Hello ${name}, 欢迎回来`;
    
    // Multiline template literals
    const multiline = `
        这是多行字符串
        第二行内容
        Third line
    `;
    
    // Tagged template literals
    const tagged = i18n`Hello ${name}`;
}

// Regular string literals
function stringDefinitions() {
    const chineseMsg = "这是一个中文消息";
    const englishMsg = "This is an English message";
    const mixedMsg = "Hello 世界";
    
    const singleQuote = '单引号字符串';
    const doubleQuote = "Double quote string";
    
    const escaped = "Line 1\nLine 2\tTabbed";
    const unicode = "Unicode: \u4e2d\u6587";
}

// UI/Display strings
function uiStrings() {
    const buttonText = "点击这里";
    const labelText = "Username";
    const placeholder = "请输入用户名";
    const tooltip = "Click to submit";
    const menuItem = "File 文件";
}

// HTTP/API related strings
function apiResponses() {
    const success = '{ "code": 200, "message": "操作成功" }';
    const error = '{ "code": 500, "message": "服务器内部错误" }';
    const notFound = '{ "code": 404, "message": "Resource not found" }';
    
    const url1 = "/api/users/中文";
    const url2 = "/api/v1/resource";
}

// SQL query strings
function sqlQueries() {
    const query1 = "SELECT * FROM users WHERE name = '王五'";
    const query2 = "INSERT INTO logs (message) VALUES ('User login')";
    const query3 = "UPDATE settings SET value = '中文配置'";
}

// Regex patterns as strings
function regexPatterns() {
    const pattern1 = /\d{4}-\d{2}-\d{2}/;
    const pattern2 = /[\u4e00-\u9fa5]+/;  // Chinese characters
    const pattern3 = /^\w+@\w+\.\w+$/;
}

// File path strings
function filePaths() {
    const path1 = "/home/user/文档/file.txt";
    const path2 = "C:\\Users\\Admin\\Documents\\file.txt";
    const path3 = "./config/设置.json";
}

// Configuration strings
function configStrings() {
    const dbHost = "localhost";
    const dbName = "生产数据库";
    const appName = "MyApplication";
    const version = "1.0.0";
}

// JSDoc with different languages
/**
 * 这是一个中文文档注释。
 * @param {string} name - 用户名
 * @returns {string} 问候语
 */
function chineseDocumentedFunction(name) {
    return `你好, ${name}`;
}

/**
 * This is an English JSDoc comment.
 * @param {string} name - User name
 * @returns {string} Greeting
 */
function englishDocumentedFunction(name) {
    return `Hello, ${name}`;
}

// Alert/Confirm strings
function dialogStrings() {
    alert("操作成功完成");
    alert("Operation completed successfully");
    
    confirm("确定要删除吗？");
    confirm("Are you sure you want to delete?");
    
    prompt("请输入您的姓名：");
    prompt("Please enter your name:");
}

// Event messages
const eventMessages = {
    click: "点击事件触发",
    load: "页面加载完成",
    error: "An error occurred"
};

// Validation messages
const validationMessages = {
    required: "此字段为必填项",
    email: "请输入有效的电子邮件地址",
    minLength: "Minimum length is 8 characters"
};

console.log("程序启动");
console.log("Program started");
