#![allow(dead_code)]
//! Synthetic Rust source generators for icb-rustc benchmarks.

/// Large flat file with many independent functions.
pub fn build_large_flat_source(num_functions: usize) -> String {
    let mut src = String::with_capacity(num_functions * 96);

    src.push_str("// synthetic large flat rust source\n\n");

    for i in 0..num_functions {
        src.push_str(&format!(
            "#[inline(always)]\nfn func_{i}() -> usize {{ {i} }}\n\n"
        ));

        if i > 0 {
            src.push_str(&format!(
                "fn caller_{i}() -> usize {{ func_{}() + func_{i}() }}\n\n",
                i - 1
            ));
        }
    }

    src.push_str("fn main() {\n");
    src.push_str("    let mut acc = 0usize;\n");

    for i in 0..num_functions.min(5000) {
        if i > 0 {
            src.push_str(&format!("    acc += caller_{i}();\n"));
        } else {
            src.push_str("    acc += func_0();\n");
        }
    }

    src.push_str("    std::hint::black_box(acc);\n");
    src.push_str("}\n");

    src
}

/// Deeply nested module hierarchy.
pub fn build_deeply_nested_source(depth: usize) -> String {
    let mut src = String::new();

    src.push_str("// synthetic deeply nested rust source\n\n");

    for d in 0..depth {
        src.push_str(&format!("mod level_{d} {{\n"));
    }

    src.push_str(
        r#"
pub struct DeepStruct {
    value: usize,
}

impl DeepStruct {
    pub fn compute(&self) -> usize {
        self.value * 2
    }
}

pub fn deepest_function() -> usize {
    let s = DeepStruct { value: 42 };
    s.compute()
}
"#,
    );

    for _ in 0..depth {
        src.push_str("}\n");
    }

    src.push_str("\nfn main() {}\n");

    src
}

/// Huge call graph inside one source file.
pub fn build_many_calls_source(num_calls: usize) -> String {
    let mut src = String::with_capacity(num_calls * 128);

    src.push_str("// synthetic many-calls rust source\n\n");

    src.push_str(
        r#"
#[inline(never)]
fn root() -> usize {
    1
}
"#,
    );

    for i in 0..num_calls {
        src.push_str(&format!(
            r#"
#[inline(never)]
fn callee_{i}() -> usize {{
    root() + {i}
}}

#[inline(never)]
fn caller_{i}() -> usize {{
    callee_{i}()
}}
"#
        ));
    }

    src.push_str("\nfn driver() -> usize {\n");
    src.push_str("    let mut total = 0usize;\n");

    for i in 0..num_calls {
        src.push_str(&format!("    total += caller_{i}();\n"));
    }

    src.push_str("    total\n");
    src.push_str("}\n\n");

    src.push_str(
        r#"
fn main() {
    std::hint::black_box(driver());
}
"#,
    );

    src
}
