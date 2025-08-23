int test_two_returns(int n) {
    if (n <= 1) {
        return n;
    }
    // Explicit return statement 
    return 42;
}

int main() {
    return test_two_returns(3);
}
