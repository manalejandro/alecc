int printf(const char* format, ...);

int add(int a, int b) {
    return a + b;
}

int main() {
    printf("Result: %d\n", add(5, 3));
    return 0;
}
