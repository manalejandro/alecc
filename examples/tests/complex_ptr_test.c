#include <stdio.h>

int main() {
    int a = 10;
    int b = 20;
    int *ptr1 = &a;
    int *ptr2 = &b;
    
    // Swap the pointer targets
    *ptr1 = *ptr1 + *ptr2;
    *ptr2 = *ptr1 - *ptr2;
    *ptr1 = *ptr1 - *ptr2;
    
    printf("Swapped values: a=%d, b=%d\n", a, b);
    return 0;
}
