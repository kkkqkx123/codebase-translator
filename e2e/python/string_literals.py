# Python string literals for testing

MESSAGE = "Hello, world!"
GREETING = "Welcome to the application"
ERROR_MSG = "An error occurred while processing"

def show_message():
    print(MESSAGE)

def handle_error():
    print("Error: operation failed")

class Example:
    def __init__(self):
        self.name = "Example class"
        
    def greet(self, name):
        return f"Hello, {name}!"
