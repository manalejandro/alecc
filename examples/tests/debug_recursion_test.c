#include <stdio.h>

int test_recursion(int n) {
    printf("Entering test_recursion with n = %d\n", n);
    if (n <= 0) {
        printf("Base case reached, returning 0\n");
        return 0;
    }
    printf("Calling recursion with n-1 = %d\n", n-1);
    int result = test_recursion(n - 1);
    printf("Recursion returned %d for n = %d\n", result, n);
    return result + 1;
}

int main() {
    printf("Starting recursion test\n");
    int result = test_recursion(2);
    printf("Final result: %d\n", result);
    return 0;
}
