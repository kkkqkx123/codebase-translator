// Go String Literals Fixture
//
// Tests extraction of string literals from various Go code patterns.

package main

import (
	"errors"
	"fmt"
	"log"
)

// Error message strings
func errorExamples() {
	panic("系统遇到致命错误，无法继续执行")
	panic("Fatal system error occurred")

	err1 := errors.New("配置加载失败")
	err2 := errors.New("Configuration loading failed")

	fmt.Errorf("用户 %s 不存在", "张三")
	fmt.Errorf("User %s not found", "John")
}

// Console/log output strings
func logExamples() {
	fmt.Println("应用程序启动成功")
	fmt.Println("Application started successfully")

	log.Println("警告：磁盘空间不足")
	log.Println("Warning: Low disk space")

	fmt.Printf("处理进度: %d%%\n", 50)
	fmt.Printf("Progress: %d%%\n", 50)
}

// Format strings
func formatExamples() {
	name := "赵六"
	age := 35

	msg1 := fmt.Sprintf("用户 %s 的年龄是 %d", name, age)
	msg2 := fmt.Sprintf("User %s is %d years old", name, age)
	msg3 := fmt.Sprintf("Processing %d of %d", 1, 10)

	// Mixed language
	msg4 := fmt.Sprintf("Hello %s, 欢迎回来", name)
}

// String variable definitions
func stringDefinitions() {
	chineseMsg := "这是一个中文消息"
	englishMsg := "This is an English message"
	mixedMsg := "Hello 世界"

	rawString := `Raw string with \n no escapes`
	rawChinese := `原始字符串：无转义
支持多行`

	const ErrorMsg = "配置错误"
	const SuccessMsg = "Operation completed"
}

// UI/Display strings
func uiStrings() {
	buttonText := "点击这里"
	labelText := "Username"
	placeholder := "请输入用户名"
	tooltip := "Click to submit"
}

// Error types with messages
type AppError struct {
	Message string
}

func (e *AppError) Error() string {
	return e.Message
}

var (
	ErrNotFound     = &AppError{Message: "找不到请求的资源"}
	ErrInvalidInput = &AppError{Message: "输入参数无效"}
	ErrTimeout      = &AppError{Message: "Connection timeout"}
)

// HTTP/API related strings
func apiResponses() {
	success := `{ "code": 200, "message": "操作成功" }`
	error := `{ "code": 500, "message": "服务器内部错误" }`
	notFound := `{ "code": 404, "message": "Resource not found" }`
}

// SQL query strings
func sqlQueries() {
	query1 := "SELECT * FROM users WHERE name = '赵六'"
	query2 := "INSERT INTO logs (message) VALUES ('User login')"
	query3 := "UPDATE settings SET value = '中文配置'"
}

// Regex patterns as strings
func regexPatterns() {
	pattern1 := `\d{4}-\d{2}-\d{2}`
	pattern2 := `[\x{4e00}-\x{9fa5}]+` // Chinese characters
	pattern3 := `^\w+@\w+\.\w+$`
}

// File path strings
func filePaths() {
	path1 := "/home/user/文档/file.txt"
	path2 := "C:\\Users\\Admin\\Documents\\file.txt"
	path3 := "./config/设置.json"
}

// Configuration strings
func configStrings() {
	dbHost := "localhost"
	dbName := "生产数据库"
	appName := "MyApplication"
	version := "1.0.0"
}

// i18n strings
var translations = map[string]string{
	"hello":      "你好",
	"goodbye":    "再见",
	"welcome":    "欢迎",
	"error":      "错误",
	"success":    "成功",
	"loading":    "Loading...",
	"saving":     "Saving...",
}

// HTTP status messages
var statusMessages = map[int]string{
	200: "请求成功",
	201: "创建成功",
	400: "请求参数错误",
	401: "未授权",
	403: "禁止访问",
	404: "资源不存在",
	500: "服务器内部错误",
}

func main() {
	fmt.Println("程序启动")
	fmt.Println("Program started")
}
