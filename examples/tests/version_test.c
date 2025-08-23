// Test the new version with advanced features
int test_compound_operators() {
    int x = 10;
    x += 5;     // 15
    x *= 2;     // 30 
    x /= 3;     // 10
    return x;
}

int test_bitwise_operators() {
    int a = 12;  // 1100 in binary
    int b = 10;  // 1010 in binary
    
    int result = 0;
    result += (a & b);    // 8   (1000)
    result += (a | b);    // 14  (1110) 
    result += (a ^ b);    // 6   (0110)
    result += (a << 1);   // 24  (11000)
    result += (a >> 1);   // 6   (110)
    
    return result; // 8 + 14 + 6 + 24 + 6 = 58
}

int main() {
    int comp_result = test_compound_operators();  // 10
    int bit_result = test_bitwise_operators();    // 58
    return comp_result + bit_result;              // 68
}
