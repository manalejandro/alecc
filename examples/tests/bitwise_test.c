#include <stdio.h>

int main() {
    int a = 12;  // 1100 in binary
    int b = 10;  // 1010 in binary
    
    printf("a = %d, b = %d\n", a, b);
    printf("a & b = %d\n", a & b);  // bitwise AND
    printf("a | b = %d\n", a | b);  // bitwise OR
    printf("a ^ b = %d\n", a ^ b);  // bitwise XOR
    printf("~a = %d\n", ~a);        // bitwise NOT
    printf("a << 2 = %d\n", a << 2); // left shift
    printf("a >> 1 = %d\n", a >> 1); // right shift
    
    return 0;
}
