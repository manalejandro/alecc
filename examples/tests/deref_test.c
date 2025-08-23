#include <stdio.h>

int main() {
    int x = 42;
    int *ptr = &x;
    int y = *ptr;
    printf("y = %d\n", y);
    return 0;
}
