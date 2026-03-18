// Go comments for testing
package main

// This is a single-line comment
const value = 42

/*
This is a multi-line comment
with multiple lines of text
*/

func test() int {
    // Another comment inside function
    return value
}

// greet returns a greeting message
// name is the person to greet
func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}
