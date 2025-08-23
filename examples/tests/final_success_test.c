#include <stdio.h>

int main() {
    printf("ALECC Test Summary\n");
    
    int arr[3];
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    printf("Arrays: arr[1] = %d (FIXED!)\n", arr[1]);
    
    int val = 42;
    int* ptr = &val;
    *ptr = 99;
    printf("Pointers: value is now %d\n", val);
    
    printf("ALECC compiler SUCCESS!\n");
    return 0;
}
