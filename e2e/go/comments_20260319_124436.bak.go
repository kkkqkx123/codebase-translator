// 转到测试评论
package main

// 这是一个单行注释
const value = 42

/*
 * 这是一个多行注释
 * 多行文本 */

func test() int {
    // 函数内部的另一个注释
    return value
}

greet 返回问候信息
name 是要问候的人
func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}
