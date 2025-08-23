#include <stdio.h>

int main() {
    int arr[3];
    
    // Test with index 1 specifically  
    printf("Testing arr[1] assignment:\n");
    
    arr[1] = 42;
    printf("After arr[1] = 42: arr[1] = %d\n", arr[1]);
    
    arr[1] = 7;
    printf("After arr[1] = 7: arr[1] = %d\n", arr[1]);
    
    arr[1] = 0;
    printf("After arr[1] = 0: arr[1] = %d\n", arr[1]);
    
    arr[1] = 1;
    printf("After arr[1] = 1: arr[1] = %d\n", arr[1]);
    
    return 0;
}
