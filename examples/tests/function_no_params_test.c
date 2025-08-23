#include <stdio.h>

int say_hello() {
    printf("Hello from function!\n");
    return 42;
}

int main() {
    printf("Before function call\n");
    int result = say_hello();
    printf("After function call, result = %d\n", result);
    return 0;
}
