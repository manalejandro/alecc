#include <stdio.h>

int main() {
    printf("=== Array Assignment Debug ===\n");
    int arr[3];
    
    printf("Before assignments:\n");
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    printf("Assigning arr[0] = 10\n");
    arr[0] = 10;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    printf("Assigning arr[1] = 20\n");
    arr[1] = 20;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    printf("Assigning arr[2] = 30\n");
    arr[2] = 30;
    printf("arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    return 0;
}
