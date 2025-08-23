#include <stdio.h>

int main() {
    int x = 42;
    int *ptr = &x;
    
    printf("x = %d\n", x);
    printf("*ptr = %d\n", *ptr);
    
    *ptr = 100;
    printf("After *ptr = 100, x = %d\n", x);
    
    return 0;
}
