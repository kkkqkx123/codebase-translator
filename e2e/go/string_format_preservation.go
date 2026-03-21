package main

import "fmt"

func testRegularStrings() {
	s1 := "Hello, world!"
	s2 := "Simple text"
	fmt.Println(s1)
}

func testRawStrings() {
	raw := `Hello, world!`
	multiline := `Line 1
Line 2
Line 3`
	fmt.Println(raw)
}

func testFormatStrings() {
	name := "Alice"
	formatted := fmt.Sprintf("Hello, %s!", name)
	fmt.Println(formatted)
}

func testErrorMessages() {
	panic("Something went wrong")
}

func main() {
	testRegularStrings()
	testRawStrings()
}
