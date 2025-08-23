#include <stdio.h>

int main() {
    printf("Testing simple loop...\n");
    
    int i = 0;
    while (i < 3) {
        printf("i = %d\n", i);
        i = i + 1;
    }
    
    printf("Loop completed!\n");
    return 0;
}
