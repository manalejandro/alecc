/*
 * ALECC Pointer Support Test Suite
 * This file demonstrates all implemented pointer features
 */

#include <stdio.h>

int main() {
    // Test 1: Basic pointer declaration and assignment  
    int x = 42;
    int *ptr = &x;  // Address-of operator
    
    // Test 2: Reading through pointer dereference
    int value = *ptr;  // Dereference for reading
    
    // Test 3: Writing through pointer dereference  
    *ptr = 100;  // Dereference for assignment
    
    // Test 4: Multiple pointers
    int y = 200;
    int *ptr2 = &y;
    
    // Test 5: Pointer reassignment
    ptr = ptr2;  // Point to different variable
    
    // Test 6: Complex expressions with dereferences
    int result = *ptr + *ptr2;  // Both pointers pointing to same location
    
    // Test 7: Null pointer
    int *null_ptr = 0;
    
    printf("All pointer tests compiled successfully!\n");
    return 0;
}
