#![allow(dead_code)]

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

pub fn build_large_flat_go(n: usize) -> String {
    let mut src = String::from("package main\n\n");
    for i in 0..n {
        src.push_str(&format!("func func{}() {{}}\n", i));
        if i % 2 == 0 {
            src.push_str(&format!("func caller{}() {{ func{}() }}\n", i, i));
        }
    }
    src
}

pub fn build_large_flat_ruby(n: usize) -> String {
    let mut src = String::new();
    for i in 0..n {
        src.push_str(&format!("def func{}\nend\n", i));
        if i % 2 == 0 {
            src.push_str(&format!("def caller{}\n  func{}\nend\n", i, i));
        }
    }
    src
}

pub fn build_deeply_nested_source(depth: usize) -> String {
    let mut src = String::new();
    for d in 0..depth {
        src.push_str(&format!("struct L{} {{\n", d));
    }
    src.push_str("void target() {}\n");
    for _ in 0..depth {
        src.push_str("};\n");
    }
    src
}

pub fn build_deeply_nested_go(depth: usize) -> String {
    let mut src = String::from("package main\n\n");
    for d in 0..depth {
        src.push_str(&format!("type Level{} struct {{\n", d));
    }
    src.push_str("func (l *Level0) Target() {}\n");
    for _ in 0..depth {
        src.push_str("}\n");
    }
    src
}

pub fn build_deeply_nested_ruby(depth: usize) -> String {
    let mut src = String::new();
    for d in 0..depth {
        src.push_str(&format!("module Level{}\n", d));
    }
    src.push_str("def target; end\n");
    for _ in 0..depth {
        src.push_str("end\n");
    }
    src
}

pub fn build_many_calls_source(num_calls: usize) -> String {
    let mut src = String::from("void callee_all() {}\n");
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

pub fn build_many_calls_go(n: usize) -> String {
    let mut src = String::from("package main\n\n");
    for i in 0..n {
        src.push_str(&format!("func callee{}() {{}}\n", i));
        src.push_str(&format!("func caller{}() {{ callee{}() }}\n", i, i));
    }
    src.push_str("func driver() { ");
    for i in 0..n {
        src.push_str(&format!("caller{}() ", i));
    }
    src.push_str("}\n");
    src
}

pub fn build_many_calls_ruby(n: usize) -> String {
    let mut src = String::new();
    for i in 0..n {
        src.push_str(&format!("def callee{}\nend\n", i));
        src.push_str(&format!("def caller{}\n  callee{}\nend\n", i, i));
    }
    src.push_str("def driver\n  ");
    for i in 0..n {
        src.push_str(&format!("caller{} ", i));
    }
    src.push_str("\nend\n");
    src
}
