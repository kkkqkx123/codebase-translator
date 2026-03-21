// Save test file in string format
// This file is used to test various JavaScript string literal types

function testRegularStrings() {
    const s1 = "Hello, world!";
    const s2 = 'Simple text';
    console.log(s1);
}

function testTemplateStrings() {
    const name = "Alice";
    const formatted = `Hello, ${name}!`;
    console.log(formatted);
}

function testMultilineStrings() {
    const multiline = `Line 1
Line 2
Line 3`;
    console.log(multiline);
}

function testErrorMessages() {
    throw new Error("Something went wrong");
    console.error("Failed to connect");
}

function testEscapedStrings() {
    const escaped = "Hello\nWorld\t!";
    const quoted = 'He said "Hello"';
    console.log(escaped);
}

module.exports = {
    testRegularStrings,
    testTemplateStrings
};
