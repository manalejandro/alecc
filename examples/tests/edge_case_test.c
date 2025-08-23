#include <stdio.h>

int main() {
    printf("=== Array Assignment Edge Cases ===\n");
    
    // Test 1: Multiple arrays
    int arr1[3];
    int arr2[3];
    arr1[0] = 1;
    arr1[1] = 2;
    arr1[2] = 3;
    arr2[0] = 10;
    arr2[1] = 20;
    arr2[2] = 30;
    printf("arr1: [%d, %d, %d]\n", arr1[0], arr1[1], arr1[2]);
    printf("arr2: [%d, %d, %d]\n", arr2[0], arr2[1], arr2[2]);
    
    // Test 2: Sequential assignments
    int seq[5];
    int k = 0;
    while (k < 5) {
        seq[k] = k * k;
        k = k + 1;
    }
    printf("Sequence: ");
    k = 0;
    while (k < 5) {
        printf("%d ", seq[k]);
        k = k + 1;
    }
    printf("\n");
    
    // Test 3: Array with function calls
    int result[3];
    printf("Setting result[0] = 5\n");
    result[0] = 5;
    printf("result[0] = %d\n", result[0]);
    printf("Setting result[1] = result[0] * 2\n");
    result[1] = result[0] * 2;
    printf("result[1] = %d\n", result[1]);
    printf("Setting result[2] = result[1] + result[0]\n");
    result[2] = result[1] + result[0];
    printf("result[2] = %d\n", result[2]);
    
    // Test 4: Large indices to test multiplication
    int big_arr[100];
    big_arr[99] = 42;
    big_arr[50] = 25;
    big_arr[1] = 99;
    printf("big_arr[99] = %d\n", big_arr[99]);
    printf("big_arr[50] = %d\n", big_arr[50]);
    printf("big_arr[1] = %d\n", big_arr[1]);
    
    printf("All edge cases passed!\n");
    return 0;
}
