use alecc::codegen::CodeGenerator;
use alecc::lexer::Lexer;
use alecc::optimizer::{OptimizationLevel, Optimizer};
use alecc::parser::Parser;
use alecc::targets::Target;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SIMPLE_C_CODE: &str = r#"
int main() {
    int x = 42;
    int y = x + 10;
    return y;
}
"#;

const COMPLEX_C_CODE: &str = r#"
#include <stdio.h>

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
"#;

fn bench_lexer(c: &mut Criterion) {
    c.bench_function("lexer_simple", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(SIMPLE_C_CODE.to_string()));
            black_box(lexer.tokenize().unwrap());
        })
    });

    c.bench_function("lexer_complex", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(COMPLEX_C_CODE.to_string()));
            black_box(lexer.tokenize().unwrap());
        })
    });
}

fn bench_parser(c: &mut Criterion) {
    let mut lexer = Lexer::new(SIMPLE_C_CODE.to_string());
    let tokens = lexer.tokenize().unwrap();

    c.bench_function("parser_simple", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(tokens.clone()));
            black_box(parser.parse().unwrap());
        })
    });
}

fn bench_codegen(c: &mut Criterion) {
    let mut lexer = Lexer::new(SIMPLE_C_CODE.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    c.bench_function("codegen_i386", |b| {
        b.iter(|| {
            let mut codegen = CodeGenerator::new(black_box(Target::I386));
            black_box(codegen.generate(&program).unwrap());
        })
    });

    c.bench_function("codegen_amd64", |b| {
        b.iter(|| {
            let mut codegen = CodeGenerator::new(black_box(Target::Amd64));
            black_box(codegen.generate(&program).unwrap());
        })
    });

    c.bench_function("codegen_arm64", |b| {
        b.iter(|| {
            let mut codegen = CodeGenerator::new(black_box(Target::Arm64));
            black_box(codegen.generate(&program).unwrap());
        })
    });
}

fn bench_optimizer(c: &mut Criterion) {
    let mut lexer = Lexer::new(SIMPLE_C_CODE.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    c.bench_function("optimizer_o0", |b| {
        b.iter(|| {
            let mut prog_copy = black_box(program.clone());
            let mut optimizer = Optimizer::new(OptimizationLevel::None);
            black_box(optimizer.optimize(&mut prog_copy).unwrap());
        })
    });

    c.bench_function("optimizer_o2", |b| {
        b.iter(|| {
            let mut prog_copy = black_box(program.clone());
            let mut optimizer = Optimizer::new(OptimizationLevel::Moderate);
            black_box(optimizer.optimize(&mut prog_copy).unwrap());
        })
    });

    c.bench_function("optimizer_o3", |b| {
        b.iter(|| {
            let mut prog_copy = black_box(program.clone());
            let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
            black_box(optimizer.optimize(&mut prog_copy).unwrap());
        })
    });
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_codegen,
    bench_optimizer
);
criterion_main!(benches);
