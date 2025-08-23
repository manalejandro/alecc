#include <stdio.h>

int main() {
    int arr[3];
    
    printf("Step 1: arr[0] = 1\n");
    arr[0] = 1;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    printf("Step 2: arr[2] = 3\n");
    arr[2] = 3;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    printf("Step 3: arr[1] = 2\n");
    arr[1] = 2;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    return 0;
}
