#include <stdio.h>

int main() {
    printf("Testing for loop...\n");
    
    int i;
    for (i = 0; i < 5; i = i + 1) {
        printf("for loop: i = %d\n", i);
    }
    
    printf("For loop completed!\n");
    return 0;
}
