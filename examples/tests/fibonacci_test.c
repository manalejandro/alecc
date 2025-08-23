#include <stdio.h>
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
int main() {
    printf("Testing fibonacci...\n");
    for (int i = 0; i <= 10; i++) {
        printf("fibonacci(%d) = %d\n", i, fibonacci(i));
    }
    return 0;
}
