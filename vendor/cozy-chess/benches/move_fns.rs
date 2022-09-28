use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use cozy_chess::*;

fn bench(criterion: &mut Criterion, id: &str, elem: usize, mut routine: impl FnMut()) {
    criterion
        .benchmark_group("move_fns")
        .throughput(Throughput::Elements(elem as u64))
        .bench_function(id, |b| b.iter(&mut routine));
}

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let mut state = 0x6D696E75736B656C76696E2062616974u128 | 1;
    let mut rand = || {
        state = state.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (state >> 122) as u32;
        let xsl = (state >> 64) as u64 ^ state as u64;
        xsl.rotate_right(rot)
    };

    let blockers = (0..1000)
        .map(|_| BitBoard(rand() & rand() & rand()))
        .collect::<Vec<_>>();

    bench(criterion, "get_rook_moves", Square::NUM * blockers.len(), || {
        for &square in black_box(&Square::ALL) {
            for &blockers in black_box(&blockers) {
                black_box(get_rook_moves(square, blockers));
            }
        }
    });

    bench(criterion, "get_bishop_moves", Square::NUM * blockers.len(), || {
        for &square in black_box(&Square::ALL) {
            for &blockers in black_box(&blockers) {
                black_box(get_bishop_moves(square, blockers));
            }
        }
    });

    bench(criterion, "get_pawn_quiets", Color::NUM * Square::NUM * blockers.len(), || {
        for &color in black_box(&Color::ALL) {
            for &square in black_box(&Square::ALL) {
                for &blockers in black_box(&blockers) {
                    black_box(get_pawn_quiets(square, color, blockers));
                }
            }
        }
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);
