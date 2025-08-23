#include <stdio.h>

int if_else_return_test_fixed(int n) {
    printf("Called with n = %d\n", n);
    if (n <= 1) {
        printf("Returning from if branch\n");
        return 1;
    } else {
        printf("In else branch\n");
        return n * 2;
    }
}

int main() {
    printf("Testing if-else return with explicit braces...\n");
    int result1 = if_else_return_test_fixed(0);
    printf("Result1 = %d\n", result1);
    int result2 = if_else_return_test_fixed(3);
    printf("Result2 = %d\n", result2);
    return 0;
}
