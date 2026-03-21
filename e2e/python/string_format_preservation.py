# 字符串格式保存测试文件
# 该文件用于测试各种 Python 字符串字面类型

def test_regular_strings():
    s1 = "Hello, world!"
    s2 = 'Simple text'
    print(s1)

def test_f_strings():
    name = "Alice"
    formatted = f"Hello, {name}!"
    print(formatted)

def test_raw_strings():
    raw1 = r"Hello\nWorld"
    raw2 = r'C:\Users\test'
    print(raw1)

def test_multiline_strings():
    multiline1 = """Line 1
Line 2
Line 3"""
    multiline2 = '''Another
multiline
string'''
    print(multiline1)

def test_error_messages():
    raise ValueError("Something went wrong")
    logging.error("Failed to connect")

def test_format_strings():
    template = "Hello, %s!"
    formatted = template % "World"
    print(formatted)

def test_unicode_strings():
    s = "Hello, 世界!"
    print(s)
