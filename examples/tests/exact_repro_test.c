#include <stdio.h>

int main() {
    int arr[3];
    arr[0] = 1;
    printf("After arr[0] = 1: arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    arr[1] = 2;
    printf("After arr[1] = 2: arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    arr[2] = 3;
    printf("After arr[2] = 3: arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    return 0;
}
