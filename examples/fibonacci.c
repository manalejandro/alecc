// Example: Fibonacci calculation
int printf(const char* format, ...);

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    int i;
    for (i = 0; i < 10; i++) {
        printf("fib(%d) = %d\n", i, fibonacci(i));
    }
    return 0;
}
