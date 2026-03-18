"""Python String Literals Fixture

Tests extraction of string literals from various Python code patterns.
"""

import logging

# Error message strings
def error_examples():
    raise ValueError("数值不能为负数")
    raise ValueError("Value must be positive")
    
    raise Exception("系统发生未知错误")
    raise RuntimeError("Connection timeout")

# Console/log output strings
def log_examples():
    print("程序开始执行")
    print("Program execution started")
    
    logging.error("数据库连接失败")
    logging.error("Failed to connect to database")
    
    logging.info("用户登录成功")
    logging.info("User logged in successfully")
    
    print("处理中...", end="")
    print("完成")

# Format strings
def format_examples():
    name = "李四"
    age = 30
    
    # f-strings
    msg1 = f"用户 {name} 今年 {age} 岁"
    msg2 = f"User {name} is {age} years old"
    msg3 = f"Processing {1} of {10}"
    
    # Mixed language
    msg4 = f"Hello {name}, 欢迎回来"
    
    # format() method
    msg5 = "用户 {} 的年龄是 {}".format(name, age)
    msg6 = "User {0} is {1} years old".format(name, age)
    msg7 = "User {name} is {age} years old".format(name=name, age=age)
    
    # % formatting
    msg8 = "用户 %s 的年龄是 %d" % (name, age)
    msg9 = "User %s is %d years old" % (name, age)

# String variable definitions
def string_definitions():
    chinese_msg = "这是一个中文消息"
    english_msg = "This is an English message"
    mixed_msg = "Hello 世界"
    
    raw_string = r"Raw string with \n no escapes"
    raw_chinese = r"原始字符串：无转义"
    
    multiline = """
    这是多行字符串
    第二行内容
    Third line
    """
    
    ERROR_MSG = "配置错误"
    SUCCESS_MSG = "Operation completed"

# UI/Display strings
def ui_strings():
    button_text = "点击这里"
    label_text = "Username"
    placeholder = "请输入用户名"
    tooltip = "Click to submit"

# Error types with messages
class AppError(Exception):
    """Application error with message."""
    pass

class ValidationError(AppError):
    """Validation failed."""
    def __init__(self):
        super().__init__("输入数据验证失败")

class NotFoundError(AppError):
    """Resource not found."""
    def __init__(self):
        super().__init__("找不到请求的资源")

# HTTP/API related strings
def api_responses():
    success = '{ "code": 200, "message": "操作成功" }'
    error = '{ "code": 500, "message": "服务器内部错误" }'
    not_found = '{ "code": 404, "message": "Resource not found" }'

# SQL query strings
def sql_queries():
    query1 = "SELECT * FROM users WHERE name = '李四'"
    query2 = "INSERT INTO logs (message) VALUES ('User login')"
    query3 = "UPDATE settings SET value = '中文配置'"

# Regex patterns as strings
def regex_patterns():
    pattern1 = r"\d{4}-\d{2}-\d{2}"
    pattern2 = r"[\u4e00-\u9fa5]+"  # Chinese characters
    pattern3 = r"^\w+@\w+\.\w+$"

# File path strings
def file_paths():
    path1 = "/home/user/文档/file.txt"
    path2 = "C:\\Users\\Admin\\Documents\\file.txt"
    path3 = "./config/设置.json"

# Configuration strings
def config_strings():
    db_host = "localhost"
    db_name = "生产数据库"
    app_name = "MyApplication"
    version = "1.0.0"

# Docstrings with different languages
def documented_function():
    """这是一个中文文档字符串。
    
    详细描述函数的功能。
    """
    pass

def english_documented_function():
    """This is an English docstring.
    
    Detailed description of the function.
    """
    pass

if __name__ == "__main__":
    print("程序启动")
    print("Program started")
