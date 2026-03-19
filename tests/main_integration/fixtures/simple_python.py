# 这是一个简单的Python文件，用于测试翻译功能
# 包含中文注释和文档字符串

def add(a, b):
    """
    计算两个数的和
    
    Args:
        a: 第一个数字
        b: 第二个数字
    
    Returns:
        两个数的和
    """
    return a + b

def subtract(a, b):
    """
    计算两个数的差
    
    Args:
        a: 被减数
        b: 减数
    
    Returns:
        两个数的差
    """
    return a - b

class Calculator:
    """简单的计算器类"""
    
    def __init__(self, name):
        """
        初始化计算器
        
        Args:
            name: 计算器名称
        """
        self.name = name
    
    def get_name(self):
        """获取计算器名称"""
        return self.name

if __name__ == "__main__":
    print("测试翻译功能")
    result = add(10, 20)
    print(f"结果: {result}")
