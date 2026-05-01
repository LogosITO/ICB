#![allow(dead_code)]

//! Synthetic source generators for benchmarks.

/// Generate `num_functions` empty functions, with every second one having a caller.
pub fn build_large_flat_source(num_functions: usize) -> String {
    let mut src = String::with_capacity(num_functions * 64);
    for i in 0..num_functions {
        src.push_str(&format!("void func{}() {{}}\n", i));
        if i % 2 == 0 {
            src.push_str(&format!("void caller{}() {{ func{}(); }}\n", i, i));
        }
    }
    src
}

/// Generate `depth` nesting levels (classes/namespaces) each containing a function.
pub fn build_deeply_nested_source(depth: usize) -> String {
    let mut src = String::new();
    for d in 0..depth {
        src.push_str(&format!("class L{} {{\n", d));
    }
    src.push_str("void target() {}\n");
    for _ in 0..depth {
        src.push_str("};\n");
    }
    src
}

/// Generate a single function containing `num_calls` direct calls to distinct callees.
pub fn build_many_calls_source(num_calls: usize) -> String {
    let mut src = String::from("void calleeX() {}\n");
    for i in 0..num_calls {
        src.push_str(&format!("void callee{}() {{}}\n", i));
        src.push_str(&format!("void caller{}() {{ callee{}(); }}\n", i, i));
    }
    src.push_str("void driver() { ");
    for i in 0..num_calls {
        src.push_str(&format!("caller{}(); ", i));
    }
    src.push_str("}\n");
    src
}

/// Generate a file that includes a standard header and defines a simple function.
pub fn build_source_with_system_include() -> &'static str {
    "#include <vector>\nvoid func() {}\n"
}
