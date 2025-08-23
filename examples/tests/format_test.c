// Simple test without stdio.h to verify formatting didn't break functionality
int main() {
    // Arrays (our main fix!)
    int arr[3];
    arr[0] = 10;
    arr[1] = 20;  // The bug we fixed
    arr[2] = 30;
    
    // Pointers
    int val = 42;
    int* ptr = &val;
    *ptr = 99;
    
    // Return the sum to verify functionality
    return arr[1] + val; // Should be 20 + 99 = 119
}
