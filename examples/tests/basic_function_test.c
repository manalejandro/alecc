#include <stdio.h>

int test_param(int x) {
    printf("  In test_param: x = %d\n", x);
    return x + 1;
}

int main() {
    printf("Testing basic function calls...\n");
    
    int result1 = test_param(5);
    printf("Result 1: %d\n", result1);
    
    int result2 = test_param(10);
    printf("Result 2: %d\n", result2);
    
    printf("Test completed.\n");
    return 0;
}
