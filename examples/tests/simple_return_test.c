#include <stdio.h>

int simple_return_test(int n) {
    printf("Called with n = %d\n", n);
    return n * 2;
}

int main() {
    printf("Testing simple return...\n");
    int result = simple_return_test(5);
    printf("Result = %d\n", result);
    return 0;
}
