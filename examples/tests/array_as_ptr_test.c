#include <stdio.h>

int main() {
    int arr[3];
    arr[0] = 10;
    int *ptr = arr;
    printf("Value: %d\n", *ptr);
    return 0;
}
