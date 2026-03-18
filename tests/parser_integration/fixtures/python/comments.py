# Python comments for testing

# This is a single-line comment
value = 42

"""
This is a multi-line comment
with multiple lines of text
"""

def test():
    # Another comment inside function
    return value

def greet(name):
    """
    Documentation string for greet function
    Args:
        name: The name to greet
    Returns:
        A greeting message
    """
    return f"Hello, {name}!"

class Example:
    """Example class for testing"""
    
    def method(self):
        """Method documentation"""
        pass
