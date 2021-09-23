use chess::{Board, ChessMove, Color};

#[derive(Debug, Clone)]
pub struct Position {
    board: Vec<Board>,
    moves: Vec<Option<ChessMove>>,
}

//TODO: Counting Bloom Filter for threefold repetition detection
impl Position {
    pub fn new(board: Board) -> Self {
        Self {
            board: vec![board],
            moves: vec![],
        }
    }

    #[inline]
    pub fn three_fold_repetition(&self) -> bool {
        let hash = self.hash();
        self.board
            .iter()
            .rev()
            .step_by(2)
            .skip(1)
            .filter(|board| board.get_hash() == hash)
            .count()
            >= 2
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.board[self.board.len() - 1]
    }
    
    #[inline]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.board.push(new_board);
            self.moves.push(None);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn turn(&self) -> i32 {
        match self.board().side_to_move() {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    #[inline]
    pub fn make_move(&mut self, make_move: ChessMove) {
        self.board.push(self.board().make_move_new(make_move));
        self.moves.push(Some(make_move));
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.board.pop();
        self.moves.pop();
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.board().get_hash()
    }
}
