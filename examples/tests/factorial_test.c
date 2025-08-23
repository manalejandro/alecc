#include <stdio.h>
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
int main() {
    printf("Testing factorial...\n");
    for (int i = 0; i <= 5; i++) {
        printf("factorial(%d) = %d\n", i, factorial(i));
    }
    return 0;
}
