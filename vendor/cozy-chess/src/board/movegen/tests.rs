use std::collections::HashSet;

use super::*;

fn perft(board: &Board, depth: u8) -> u64 {
    let mut nodes = 0;
    match depth {
        0 => nodes += 1,
        1 => {
            board.generate_moves(|moves| {
                nodes += moves.len() as u64;
                false
            });
        }
        _ => {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    let child_nodes = perft(&board, depth - 1);
                    nodes += child_nodes;
                }
                false
            });
        }
    }
    nodes
}

macro_rules! make_perft_test {
    ($($name:ident($board:expr $(,$node:expr)*);)*) => {
        $(#[test]
        fn $name() {
            let board = $board.parse::<Board>()
                .or_else(|_| Board::from_fen($board, true))
                .unwrap();
            const NODES: &'static [u64] = &[$($node),*];
            for (depth, &nodes) in NODES.iter().enumerate() {
                assert_eq!(perft(&board, depth as u8), nodes, "Perft {}", depth);
            }
        })*
    };
}

make_perft_test! {
    perft_startpos(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        1,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324
    );
    perft_kiwipete(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        1,
        48,
        2039,
        97862,
        4085603,
        193690690
    );
    perft_position_3(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        1,
        14,
        191,
        2812,
        43238,
        674624,
        11030083,
        178633661
    );
    perft_position_4(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        1,
        6,
        264,
        9467,
        422333,
        15833292
    );
    perft_position_5(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        1,
        44,
        1486,
        62379,
        2103487,
        89941194
    );
    perft_position_6(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        1,
        46,
        2079,
        89890,
        3894594,
        164075551
    );
    perft_960_position_333(
        "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9",
        1,
        29,
        502,
        14569,
        287739,
        8652810,
        191762235
    );
    perft_960_position_404(
        "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9",
        1,
        27,
        916,
        25798,
        890435,
        26302461,
        924181432
    );
    perft_960_position_789(
        "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9",
        1,
        24,
        600,
        15347,
        408207,
        11029596,
        308553169
    );
    perft_960_position_726(
        "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9",
        1,
        29,
        861,
        24504,
        763454,
        22763215,
        731511256
    );
}

#[test]
fn subset_movegen_kiwipete() {
    fn visit(board: &Board, depth: u8) {
        let random = board.hash();
        let subset_a = BitBoard(random);
        let subset_b = !subset_a;
        let mut subset_moves = 0;
        board.generate_moves_for(subset_a, |moves| {
            subset_moves += moves.len();
            false
        });
        board.generate_moves_for(subset_b, |moves| {
            subset_moves += moves.len();
            false
        });
        let mut total_moves = 0;
        board.generate_moves(|moves| {
            total_moves += moves.len();
            false
        });
        assert_eq!(subset_moves, total_moves);
        if depth > 0 {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    visit(&board, depth - 1);
                }
                false
            });
        }
    }
    let board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        .parse()
        .unwrap();
    visit(&board, 4);
}

fn test_is_legal(board: Board) {
    let mut legals = HashSet::new();
    board.generate_moves(|mvs| {
        legals.extend(mvs);
        false
    });

    const PROMOS: [Option<Piece>; 7] = [
        None,
        Some(Piece::Pawn),
        Some(Piece::Knight),
        Some(Piece::Bishop),
        Some(Piece::Rook),
        Some(Piece::Queen),
        Some(Piece::King),
    ];

    for from in Square::ALL {
        for to in Square::ALL {
            for promotion in PROMOS {
                let mv = Move {
                    from,
                    to,
                    promotion,
                };
                assert_eq!(legals.contains(&mv), board.is_legal(mv), "{}", mv);
            }
        }
    }
}

#[test]
fn legality_simple() {
    test_is_legal(Board::default());
    test_is_legal(
        "rk2r3/pn1p1p1p/1p4NB/2pP1K2/4p2N/1P3BP1/P1P3PP/1R3q2 w - - 2 32"
            .parse()
            .unwrap(),
    );
}

#[test]
fn legality_castles() {
    test_is_legal(
        "rnbqk2r/ppppbp1p/5np1/4p3/4P3/3P1N2/PPP1BPPP/RNBQK2R w KQkq - 0 5"
            .parse()
            .unwrap(),
    );
    test_is_legal(
        "rnbqk2r/ppppbp1p/5npB/4p3/4P3/3P1N2/PPP1BPPP/RN1QK2R b KQkq - 1 5"
            .parse()
            .unwrap(),
    );
    test_is_legal(
        "r1bqk2r/ppppbp1p/2n2npB/4p3/4P3/2NP1N2/PPPQBPPP/R3K2R w KQq - 6 8"
            .parse()
            .unwrap(),
    );
    test_is_legal(
        "r1bqk2r/ppppbp1p/2n2npB/4p3/4P3/2NP1N2/PPPQBPPP/R2K3R b q - 7 8"
            .parse()
            .unwrap(),
    );
    test_is_legal(
        "rnbqkbn1/pppprppp/8/8/8/8/PPPP1PPP/RNBQK2R w KQq - 0 1"
            .parse()
            .unwrap(),
    );
}

#[test]
fn legality_castles_960() {
    test_is_legal(
        Board::from_fen(
            "rq1kr3/p1ppbp1p/bpn3pB/3Np3/3P4/1P1Q1Nn1/P1P1BPPP/R2KR3 w AEae - 3 15",
            true,
        )
        .unwrap(),
    );
    test_is_legal(
        Board::from_fen(
            "rq1kr3/p1ppbp1p/bpn3pB/3Np3/3P4/1P1Q1Nn1/P1P1BPPP/R2KR3 b AEae - 3 15",
            true,
        )
        .unwrap(),
    );
    test_is_legal(
        Board::from_fen(
            "rk2r3/pqppbp1p/bpn3pB/3Npn2/3P4/1P1Q1N2/P1P2PPP/RKRB4 w ACa - 3 15",
            true,
        )
        .unwrap(),
    );
}

#[test]
fn legality_en_passant() {
    test_is_legal(
        "rk2r3/pn1p1p1p/1p4NB/q1pP1K2/4p2b/1P3NP1/P1P3PP/R1RB4 w - - 0 29"
            .parse()
            .unwrap(),
    );
    test_is_legal(
        "rk2r3/pn1p1p1p/1p4NB/2pP1K2/4p2N/qP4P1/P1P3PP/R1RB4 w - c6 0 30"
            .parse()
            .unwrap(),
    );
}
