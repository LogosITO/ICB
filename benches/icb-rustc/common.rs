//! Synthetic Rust source generators for icb-rustc benchmarks.

pub fn build_large_flat_source(num_functions: usize) -> String {
    let mut src = String::with_capacity(num_functions * 64);
    for i in 0..num_functions {
        src.push_str(&format!("fn func{}() {{}}\n", i));
        if i % 2 == 0 {
            src.push_str(&format!("fn caller{}() {{ func{}(); }}\n", i, i));
        }
    }
    src
}

pub fn build_deeply_nested_source(depth: usize) -> String {
    let mut src = String::new();
    for d in 0..depth {
        src.push_str(&format!("struct L{} {{\n", d));
    }
    src.push_str("fn target() {}\n");
    for _ in 0..depth {
        src.push_str("}\n");
    }
    src
}

pub fn build_many_calls_source(num_calls: usize) -> String {
    let mut src = String::from("fn calleeX() {}\n");
    for i in 0..num_calls {
        src.push_str(&format!("fn callee{}() {{}}\n", i));
        src.push_str(&format!("fn caller{}() {{ callee{}(); }}\n", i, i));
    }
    src.push_str("fn driver() { ");
    for i in 0..num_calls {
        src.push_str(&format!("caller{}(); ", i));
    }
    src.push_str("}\n");
    src
}
