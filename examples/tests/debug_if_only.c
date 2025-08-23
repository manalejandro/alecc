#include <stdio.h>

int debug_if_only(int n) {
    if (n > 0) {
        printf("In if branch\n");
        return n + 1;
    }
    return 0;
}

int main() {
    printf("Testing if-only...\n");
    int result = debug_if_only(5);
    printf("Result = %d\n", result);
    return 0;
}
