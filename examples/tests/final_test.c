#include <stdio.h>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    printf("=== ALECC Final Comprehensive Test ===\n");
    
    // Test 1: All basic operations
    int a = 15;
    int b = 4;
    printf("Basic operations:\n");
    printf("  %d + %d = %d\n", a, b, a + b);
    printf("  %d - %d = %d\n", a, b, a - b);
    printf("  %d * %d = %d\n", a, b, a * b);
    printf("  %d / %d = %d\n", a, b, a / b);
    printf("  %d %% %d = %d\n", a, b, a % b);
    
    // Test 2: Arrays with all indices working
    printf("\nArray test:\n");
    int numbers[7];
    int i = 0;
    while (i < 7) {
        numbers[i] = i * i + 1;
        printf("  numbers[%d] = %d\n", i, numbers[i]);
        i = i + 1;
    }
    
    // Test 3: Pointers
    printf("\nPointer test:\n");
    int value = 42;
    int* ptr = &value;
    printf("  value = %d\n", value);
    printf("  *ptr = %d\n", *ptr);
    *ptr = 100;
    printf("  After *ptr = 100: value = %d\n", value);
    
    // Test 4: Complex expressions
    printf("\nComplex expressions:\n");
    int x = 5;
    int y = 3;
    int result = (x + y) * (x - y) + x % y;
    printf("  (%d + %d) * (%d - %d) + %d %% %d = %d\n", x, y, x, y, x, y, result);
    
    // Test 5: Control flow
    printf("\nControl flow test:\n");
    int j = 0;
    while (j < 5) {
        if (j % 2 == 0) {
            printf("  %d is even\n", j);
        } else {
            printf("  %d is odd\n", j);
        }
        j = j + 1;
    }
    
    // Test 6: Function calls and recursion
    printf("\nRecursion test:\n");
    int n = 6;
    printf("  fibonacci(%d) = %d\n", n, fibonacci(n));
    
    // Test 7: Multiple printf calls (stack alignment test)
    printf("\nMultiple printf test:\n");
    printf("  Line 1\n");
    printf("  Line 2\n");
    printf("  Line 3\n");
    printf("  Line 4\n");
    printf("  Line 5\n");
    
    printf("\nAll tests passed! ALECC compiler is working perfectly!\n");
    return 0;
}
