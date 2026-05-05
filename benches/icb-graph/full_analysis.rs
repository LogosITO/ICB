//! Benchmark: full pipeline – building graph + resolving calls + collecting metrics.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::analysis;
use icb_graph::builder::GraphBuilder;
use icb_parser::facts::RawNode;

fn make_graph(size: usize) -> icb_graph::graph::CodePropertyGraph {
    let mut facts = Vec::new();
    for i in 0..size {
        facts.push(RawNode {
            language: icb_common::Language::CppTreeSitter,
            kind: NodeKind::Function,
            name: Some(format!("func{}", i)),
            usr: None,
            start_line: i + 1,
            start_col: 0,
            end_line: i + 1,
            end_col: 10,
            children: vec![],
            source_file: Some("bench.cpp".into()),
        });
        if i % 2 == 0 && i + 1 < size {
            facts.push(RawNode {
                language: icb_common::Language::CppTreeSitter,
                kind: NodeKind::CallSite,
                name: Some(format!("func{}", i + 1)),
                usr: None,
                start_line: i + 10,
                start_col: 0,
                end_line: i + 10,
                end_col: 15,
                children: vec![],
                source_file: Some("bench.cpp".into()),
            });
        }
    }
    let mut builder = GraphBuilder::new();
    builder.ingest_file_facts(&facts);
    builder.resolve_calls();
    builder.cpg
}

fn bench_full_analysis(c: &mut Criterion) {
    for &size in &[100, 500, 2000] {
        let cpg = make_graph(size);
        c.bench_function(&format!("full_analysis_{}_functions", size), |b| {
            b.iter(|| {
                analysis::collect_function_metrics(black_box(&cpg));
                analysis::detect_call_cycles(black_box(&cpg));
            })
        });
    }
}

criterion_group!(benches, bench_full_analysis);
criterion_main!(benches);
