#include <stdio.h>

int main() {
    int x = 42;
    int *ptr = &x;
    printf("Before: x = %d, *ptr = %d\n", x, *ptr);
    return 0;
}
