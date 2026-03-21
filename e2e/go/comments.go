// Go to Test Reviews
package main

// This is a one-line comment
const value = 42

/*
This is a multi-line comment
Multi-line text
*/

func test() int {
    // Another comment inside the function
    return value
}

// greet returns a greeting message
// name is the person to greet
func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}
