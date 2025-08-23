#include <stdio.h>

int countdown(int n) {
    if (n <= 0) {
        return 0;
    }
    printf("n = %d\n", n);
    return countdown(n - 1);
}

int main() {
    countdown(3);
    return 0;
}
