#include <stdio.h>

int main() {
    int x = 42;
    int *ptr = &x;
    *ptr = 100;
    printf("x = %d\n", x);
    return 0;
}
