#include <stdio.h>

int main() {
    int x = 5;
    int y = 3;
    
    printf("x = %d, y = %d\n", x, y);
    
    int sum = x + y;
    printf("x + y = %d\n", sum);
    
    int diff = x - y;
    printf("x - y = %d\n", diff);
    
    int prod = sum * diff;
    printf("(x + y) * (x - y) = %d\n", prod);
    
    int mod = x % y;
    printf("x %% y = %d\n", mod);
    
    int result = prod + mod;
    printf("result = %d\n", result);
    
    return 0;
}
