#include <stdio.h>

int main() {
    printf("=== ALECC Compiler Test Suite ===\n");
    
    // Test 1: Basic arithmetic and variables
    printf("Test 1: Basic operations\n");
    int a = 10;
    int b = 20;
    printf("  a = %d, b = %d\n", a, b);
    printf("  a + b = %d\n", a + b);
    printf("  a * b = %d\n", a * b);
    printf("  a %% 3 = %d\n", a % 3);
    
    // Test 2: Pointer operations
    printf("Test 2: Pointer operations\n");
    int x = 42;
    int *ptr = &x;
    printf("  x = %d\n", x);
    printf("  *ptr = %d\n", *ptr);
    *ptr = 100;
    printf("  After *ptr = 100: x = %d\n", x);
    
    // Test 3: Arrays
    printf("Test 3: Array operations\n");
    int arr[3];
    arr[0] = 1;
    arr[1] = 2; 
    arr[2] = 3;
    printf("  arr[0] = %d, arr[1] = %d, arr[2] = %d\n", arr[0], arr[1], arr[2]);
    
    // Test 4: Control flow
    printf("Test 4: Control flow\n");
    for (int i = 0; i < 3; i++) {
        if (i % 2 == 0) {
            printf("  %d is even\n", i);
        } else {
            printf("  %d is odd\n", i);
        }
    }
    
    // Test 5: While loops
    printf("Test 5: While loop\n");
    int count = 0;
    while (count < 3) {
        printf("  count = %d\n", count);
        count++;
    }
    
    printf("=== All tests completed successfully! ===\n");
    return 0;
}
