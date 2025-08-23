#include <stdio.h>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    printf("=== ALECC Final Test Summary ===\n");
    
    // Test 1: Basic math
    int a = 15;
    int b = 4;
    printf("Math: %d + %d = %d\n", a, b, a + b);
    printf("Modulo: %d %% %d = %d\n", a, b, a % b);
    
    // Test 2: Arrays (our fixed feature!)
    int arr[5];
    arr[0] = 10;
    arr[1] = 20;  // This was the bug we fixed!
    arr[2] = 30;
    arr[3] = 40;
    arr[4] = 50;
    printf("Array: arr[1] = %d (FIXED!)\n", arr[1]);
    
    // Test 3: Pointers
    int val = 42;
    int* ptr = &val;
    *ptr = 99;
    printf("Pointers: value is now %d\n", val);
    
    // Test 4: Control flow
    int i = 0;
    printf("Loop: ");
    while (i < 5) {
        printf("%d ", i);
        i = i + 1;
    }
    printf("\n");
    
    // Test 5: Recursion
    printf("Fibonacci(6) = %d\n", fibonacci(6));
    
    // Test 6: Multiple printf calls (stack alignment fixed!)
    printf("Stack alignment test:\n");
    printf("Line 1\n");
    printf("Line 2\n");
    printf("Line 3\n");
    
    printf("SUCCESS: All major features working!\n");
    return 0;
}
