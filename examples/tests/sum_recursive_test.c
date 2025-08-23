#include <stdio.h>
int sum_recursive(int n) {
    if (n <= 0) {
        return 0;
    }
    return n + sum_recursive(n - 1);
}
int main() {
    printf("Testing recursive sum...\n");
    for (int i = 0; i <= 10; i++) {
        printf("sum(1 to %d) = %d\n", i, sum_recursive(i));
    }
    return 0;
}
