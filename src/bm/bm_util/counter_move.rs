use cozy_chess::{Board, Color, Move};

use crate::bm::bm_runner::ab_runner::MoveData;

use super::table_types::{new_piece_to_table, PieceTo};

#[derive(Debug, Clone)]
pub struct MoveTable {
    counter_move: Box<[PieceTo<Option<Move>>; Color::NUM]>,
    followup_move: Box<[PieceTo<Option<Move>>; Color::NUM]>,
}

impl MoveTable {
    pub fn new() -> Self {
        Self {
            counter_move: Box::new([new_piece_to_table(None); Color::NUM]),
            followup_move: Box::new([new_piece_to_table(None); Color::NUM]),
        }
    }

    pub fn get_counter_move(&self, color: Color, opp_move: Option<MoveData>) -> Option<Move> {
        opp_move
            .map(|opp_move| {
                self.counter_move[color as usize][opp_move.piece as usize][opp_move.to as usize]
            })
            .unwrap_or(None)
    }

    pub fn get_followup_move(&self, color: Color, prev_move: Option<MoveData>) -> Option<Move> {
        prev_move
            .map(|prev_move| {
                self.followup_move[color as usize][prev_move.piece as usize][prev_move.to as usize]
            })
            .unwrap_or(None)
    }

    pub fn cutoff(
        &mut self,
        board: &Board,
        opp_move: Option<MoveData>,
        prev_move: Option<MoveData>,
        cutoff_move: Move,
        amt: u32,
    ) {
        if amt > 20 {
            return;
        }
        let color = board.side_to_move();
        opp_move.map(|opp_move| {
            self.counter_move[color as usize][opp_move.piece as usize][opp_move.to as usize] =
                Some(cutoff_move);
        });
        prev_move.map(|prev_move| {
            self.followup_move[color as usize][prev_move.piece as usize][prev_move.to as usize] =
                Some(cutoff_move);
        });
    }
}
