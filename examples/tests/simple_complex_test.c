#include <stdio.h>

int main() {
    int x = 5;
    int y = 3;
    int result = (x + y) * (x - y) + x % y;
    
    printf("Complex expression result: %d\n", result);
    
    return 0;
}
