// Go comments fixture for testing
//
// This file contains various Go comments for testing extraction.

package main

import "fmt"

// Package level comment

// Person represents a person with a name and age
type Person struct {
	// Name is the person's name
	Name string
	// Age is the person's age
	Age int
}

// NewPerson creates a new Person with the given name and age
//
// Parameters:
//   - name: The person's name
//   - age: The person's age
//
// Returns:
//   - A pointer to the new Person
//
// Example:
//
//	person := NewPerson("Alice", 30)
func NewPerson(name string, age int) *Person {
	return &Person{
		Name: name,
		Age:  age,
	}
}

// Greet returns a greeting message
func (p *Person) Greet() string {
	// Create greeting
	return fmt.Sprintf("Hello, my name is %s", p.Name)
}

/*
Block comment
spanning multiple lines
This is another paragraph
*/

func main() {
	// Line comment in function
	fmt.Println("Hello, World!")

	/*
		Indented block comment
		Inside a function
	*/

	// TODO: This should be filtered
	// FIXME: Fix this
	// NOTE: Important note

	// Normal comment that should be translated
	x := 5 // Inline comment
	_ = x
}

// Interface with documentation
type Greeter interface {
	// Greet returns a greeting
	Greet() string
}

// Constant with documentation
const (
	// DefaultName is the default name
	DefaultName = "Anonymous"
	// DefaultAge is the default age
	DefaultAge = 0
)

// Error handling example
func process() error {
	// Check condition
	if true {
		return fmt.Errorf("error message") // Error comment
	}
	return nil
}
