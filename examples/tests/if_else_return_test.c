#include <stdio.h>

int if_else_return_test(int n) {
    printf("Called with n = %d\n", n);
    if (n <= 1) {
        printf("Returning from if branch\n");
        return 1;
    }
    printf("In else branch\n");
    return n * 2;
}

int main() {
    printf("Testing if-else return...\n");
    int result1 = if_else_return_test(0);
    printf("Result1 = %d\n", result1);
    int result2 = if_else_return_test(3);
    printf("Result2 = %d\n", result2);
    return 0;
}
