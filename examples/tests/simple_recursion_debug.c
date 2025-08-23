#include <stdio.h>

int factorial_simple(int n) {
    printf("factorial_simple called with n = %d\n", n);
    if (n <= 1) {
        printf("Base case reached, returning 1\n");
        return 1;
    }
    printf("Calling factorial_simple(%d)\n", n - 1);
    int result = factorial_simple(n - 1);
    printf("Got result %d, returning %d * %d = %d\n", result, n, result, n * result);
    return n * result;
}

int main() {
    printf("Testing simple recursion...\n");
    int result = factorial_simple(3);
    printf("factorial_simple(3) = %d\n", result);
    return 0;
}
