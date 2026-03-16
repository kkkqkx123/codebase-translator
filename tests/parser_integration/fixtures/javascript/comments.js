/**
 * JavaScript comments fixture for testing
 *
 * This file contains various JavaScript comments for testing extraction.
 */

// Module level comment

/**
 * Adds two numbers
 * @param {number} a - First number
 * @param {number} b - Second number
 * @returns {number} Sum of a and b
 * @example
 * const result = add(1, 2);
 * console.log(result); // 3
 */
function add(a, b) {
    // Add the numbers
    return a + b;
}

/**
 * Person class
 * @class
 * @classdesc Represents a person
 */
class Person {
    /**
     * Creates a new Person
     * @param {string} name - The person's name
     * @param {number} age - The person's age
     */
    constructor(name, age) {
        /** @type {string} */
        this.name = name;
        /** @type {number} */
        this.age = age;
    }

    /**
     * Greets the person
     * @returns {string} Greeting message
     */
    greet() {
        // Return greeting
        return `Hello, my name is ${this.name}`;
    }
}

// Regular function comment
function regularFunction() {
    // Line comment
    const x = 5; // Inline comment
    return x;
}

/*
 * Block comment
 * spanning multiple lines
 */

/**
 * @typedef {Object} Config
 * @property {string} name - Configuration name
 * @property {number} value - Configuration value
 */

// TODO: This should be filtered
// FIXME: Fix this
// NOTE: Important note
// XXX: Review this

// Normal comment that should be translated

// Template literals
const name = "world";
const message = `Hello, ${name}!`;
const multiline = `
    This is a
    multiline string
`;

// String with escapes
const escaped = "Line 1\nLine 2\tTabbed";

// Regular expressions
const regex = /test pattern/g;
