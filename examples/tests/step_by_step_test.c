#include <stdio.h>

int main() {
    int x = 42;
    printf("x = %d\n", x);
    
    int *ptr = &x;
    printf("*ptr = %d\n", *ptr);
    
    *ptr = 100;
    printf("x after assignment = %d\n", x);
    
    return 0;
}
