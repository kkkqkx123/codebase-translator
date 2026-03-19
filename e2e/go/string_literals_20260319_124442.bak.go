// 用于测试的 Go 字符串字面量
package main

import "fmt"

func main() {
    message := "Hello, world!"
    greeting := "Welcome to the application"
    errorMsg := "An error occurred while processing"
    
    fmt.Println(message)
}

func showMessage() {
    fmt.Println("Displaying message")
}

func handleError() {
    fmt.Println("Error: operation failed")
}
