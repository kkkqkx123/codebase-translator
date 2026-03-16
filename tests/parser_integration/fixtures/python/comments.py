"""Python comments fixture for testing.

This module contains various Python comments and docstrings for testing extraction.
"""

# Module level comment

import os


# Function with regular comments
def add(a, b):
    # Add two numbers
    return a + b


def function_with_docstring():
    """Function with simple docstring."""
    pass


def function_with_full_docstring(param1, param2):
    """Function with comprehensive docstring.
    
    This function demonstrates a full docstring with all sections.
    
    Args:
        param1: First parameter description
        param2: Second parameter description
        
    Returns:
        Return value description
        
    Raises:
        ValueError: When something goes wrong
        
    Examples:
        >>> function_with_full_docstring(1, 2)
        3
    """
    return param1 + param2


class MyClass:
    """Class docstring.
    
    This class demonstrates class-level documentation.
    
    Attributes:
        name: The name attribute
        value: The value attribute
    """
    
    def __init__(self, name):
        """Initialize the class.
        
        Args:
            name: The name to use
        """
        self.name = name  # Instance variable
        self.value = 0  # Another instance variable
    
    def method(self):
        """Method docstring."""
        # Method comment
        pass


# TODO: This should be filtered
# FIXME: Fix this issue
# NOTE: Important note
# XXX: Review this

# Normal comment that should be translated
# This is a regular comment

# Inline comment after code
x = 5  # Set x to 5

# Multi-line comment using multiple single-line comments
# Line 1 of comment
# Line 2 of comment
# Line 3 of comment

"""
This is a module-level string literal.
It might be used as documentation.
"""

# f-strings with various formats
def format_examples():
    name = "world"
    value = 42
    
    f"Hello, {name}!"
    f"Value: {value}"
    f"Expression: {value + 1}"
    f"Format: {value:.2f}"
    f"Alignment: {value:>10}"
