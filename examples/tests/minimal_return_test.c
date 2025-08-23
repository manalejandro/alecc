#include <stdio.h>

int minimal_return_test(int n) {
    printf("Called with n = %d\n", n);
    printf("About to return\n");
    return n * 2;
}

int main() {
    printf("Testing minimal return...\n");
    int result = minimal_return_test(3);
    printf("Result = %d\n", result);
    return 0;
}
