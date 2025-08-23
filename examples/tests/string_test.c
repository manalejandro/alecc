#include <stdio.h>

int main() {
    char message[] = "Hello, World!";
    char greeting[] = "Hi there";
    
    printf("Message: %s\n", message);
    printf("Greeting: %s\n", greeting);
    
    // Test character access
    printf("First char of message: %c\n", message[0]);
    printf("Last char of greeting: %c\n", greeting[7]);
    
    return 0;
}
