use criterion::{criterion_group, criterion_main, Criterion};

use cypher_core::{
    affix::{AffixDefinitionDatabase, AffixGenerationCriteria},
    data::DataDefinitionDatabase,
};

fn criterion_benchmark(c: &mut Criterion) {
    let affix_database = AffixDefinitionDatabase::initialize();
    c.bench_function("generate affix", |b| {
        b.iter(|| {
            affix_database.generate(&AffixGenerationCriteria::default());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
