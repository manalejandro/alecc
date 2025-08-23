#include <stdio.h>

int main() {
    int x = 42;
    int *ptr = &x;
    int **ptr_to_ptr = &ptr;
    
    printf("x = %d\n", x);
    printf("&x = %p\n", &x);
    printf("ptr = %p\n", ptr);
    printf("*ptr = %d\n", *ptr);
    printf("**ptr_to_ptr = %d\n", **ptr_to_ptr);
    
    // Test pointer arithmetic
    int arr[3] = {10, 20, 30};
    int *p = arr;
    printf("arr[0] = %d, *p = %d\n", arr[0], *p);
    p++;
    printf("arr[1] = %d, *p = %d\n", arr[1], *p);
    
    return 0;
}
