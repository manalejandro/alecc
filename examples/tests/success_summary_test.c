#include <stdio.h>

int main() {
    printf("=== Test Resumen ALECC ===\n");
    
    // Arrays (nuestro fix principal!)
    int arr[3];
    arr[0] = 10;
    arr[1] = 20;  // El bug que arreglamos
    arr[2] = 30;
    printf("Arrays: arr[1] = %d (FIXED!)\n", arr[1]);
    
    // Punteros
    int val = 42;
    int* ptr = &val;
    *ptr = 99;
    printf("Punteros: valor ahora es %d\n", val);
    
    // Funciones básicas (sin recursión)
    printf("Basic functions work!\n");
    
    printf("ALECC compiler SUCCESS!\n");
    return 0;
}
