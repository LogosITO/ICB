use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::builder::GraphBuilder;
use icb_parser::facts::RawNode;

fn make_facts(size: usize) -> Vec<RawNode> {
    let mut f = Vec::new();
    for i in 0..size {
        f.push(RawNode {
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
            f.push(RawNode {
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
    f
}

fn bench_resolve_calls(c: &mut Criterion) {
    for &size in &[100, 500, 2000] {
        let facts = make_facts(size);
        c.bench_function(&format!("resolve_calls_{}_functions", size), |b| {
            b.iter(|| {
                let mut builder = GraphBuilder::new();
                builder.ingest_file_facts(black_box(&facts));
                builder.resolve_calls();
            })
        });
    }
}

criterion_group!(benches, bench_resolve_calls);
criterion_main!(benches);
