use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::analysis;
use icb_graph::graph::CodePropertyGraph;

fn make_test_graph(size: usize) -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();
    for i in 0..size {
        cpg.graph.add_node(icb_graph::graph::Node {
            kind: NodeKind::Function,
            name: Some(format!("func{}", i)),
            usr: Some("bench.cpp".into()),
            start_line: i + 1,
            end_line: i + 1,
        });
    }
    cpg
}

fn bench(c: &mut Criterion) {
    for &size in &[100, 500, 2000] {
        let graph = make_test_graph(size);
        c.bench_function(&format!("analytics_{}_functions", size), |b| {
            b.iter(|| {
                // Use the public analysis functions available in icb_graph
                let _ = analysis::detect_call_cycles(black_box(&graph));
                let _ = analysis::detect_dead_code(black_box(&graph), &["main".to_string()]);
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
