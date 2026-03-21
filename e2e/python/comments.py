# Python annotations for testing

# This is a one-line comment
value = 42

"""
This is a multi-line comment
Multi-line text
"""

def test():
    # Another comment inside the function
    return value

def greet(name):
    """
    Documentation string for the greet function
            parameters of the greet function:
                name: the name to be greeted
            return return return value value value value
                Greeting message
    """
    return f"Hello, {name}!"

class Example:
    """Test Sample Classes"""
    
    def method(self):
        """methodology documentation"""
        pass
