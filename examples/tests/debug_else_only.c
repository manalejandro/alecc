#include <stdio.h>

int debug_else_only(int n) {
    printf("Called with n = %d\n", n);
    if (n <= 1) {
        return 1;
    } else {
        return n * 2;
    }
}

int main() {
    printf("Testing else-only...\n");
    int result = debug_else_only(5);
    printf("Result = %d\n", result);
    return 0;
}
