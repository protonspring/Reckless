use super::{Board, BoardObserver};
use crate::types::{Move, MoveKind, Piece, PieceType, Square, ZOBRIST};

impl Board {
    pub fn make_null_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.state_stack.push(self.state);

        self.state.key ^= ZOBRIST.side ^ ZOBRIST.castling[self.state.castling];
        self.state.plies_from_null = 0;
        self.state.repetition = 0;
        self.state.captured = None;
        self.state.recapture_square = Square::None;

        self.update_threats();
        self.update_king_threats();

        if self.state.en_passant != Square::None {
            self.state.key ^= ZOBRIST.en_passant[self.state.en_passant];
            self.state.en_passant = Square::None;
        }
    }

    pub fn undo_null_move(&mut self) {
        self.side_to_move = !self.side_to_move();
        self.state = self.state_stack.pop().unwrap();
    }

    pub fn make_move<T: BoardObserver>(&mut self, mv: Move, observer: &mut T) {
        let from = mv.from();
        let to = mv.to();
        let mover = self.piece_on(from);
        let mover_type = mover.piece_type();
        let stm = self.side_to_move();

        self.state_stack.push(self.state);
        self.state.key ^= ZOBRIST.castling[self.state.castling] ^ ZOBRIST.side;

        if self.en_passant() != Square::None {
            self.state.key ^= ZOBRIST.en_passant[self.state.en_passant];
            self.state.en_passant = Square::None;
        }

        self.state.captured = None;
        self.state.recapture_square = Square::None;
        self.state.halfmove_clock += 1;
        self.state.plies_from_null += 1;

        if mv.is_castling() {
            self.update_hash(mover, from);
            self.update_hash(mover, to);

            let (rook_from, rook_to) = self.get_castling_rook(to);
            let rook = Piece::new(stm, PieceType::Rook);

            self.remove_piece(rook, rook_from);
            observer.on_piece_change(self, rook, rook_from, false);

            self.remove_piece(mover, from);
            self.add_piece(mover, to);
            observer.on_piece_move(self, mover, from, to);

            self.add_piece(rook, rook_to);
            observer.on_piece_change(self, rook, rook_to, true);

            self.update_hash(rook, rook_from);
            self.update_hash(rook, rook_to);
        } else {

            if mv.is_capture() {
                let captured_to = if mv.is_en_passant() { to ^ 8 } else { to };
                let captured = self.piece_on(captured_to);
                self.state.halfmove_clock = 0;
                self.remove_piece(captured, captured_to);
                observer.on_piece_change(self, captured, captured_to, false);
                self.update_hash(captured, captured_to);
                self.state.material -= captured.value();

                if !mv.is_en_passant() {
                    self.state.captured = Some(captured);
                    self.state.recapture_square = to;
                }
            }

            self.remove_piece(mover, from);
            self.add_piece(mover, to);
            observer.on_piece_move(self, mover, from, to);
            self.update_hash(mover, from);
            self.update_hash(mover, to);

            // Special pawn rules
            if mover_type == PieceType::Pawn {

                if mv.is_promotion() {
                    let promotion = Piece::new(stm, mv.promotion_piece().unwrap());
                    self.remove_piece(mover, to);
                    self.add_piece(promotion, to);
                    observer.on_piece_mutate(self, mover, promotion, to);
                    self.update_hash(mover, to);
                    self.update_hash(promotion, to);
                    self.state.material += promotion.value() - PieceType::Pawn.value();
                } else if mv.is_double_push() {
                    self.state.en_passant = Square::new((from as u8 + to as u8) / 2);
                    self.state.key ^= ZOBRIST.en_passant[self.state.en_passant];
                }

                self.state.halfmove_clock = 0;
            }
        }

        self.side_to_move = !self.side_to_move;
        self.state.castling.raw &= self.castling_rights[from] & self.castling_rights[to];
        self.state.key ^= ZOBRIST.castling[self.state.castling];

        self.update_threats();
        self.update_king_threats();
        self.update_en_passant();

        self.state.repetition = 0;

        let end = self.state.plies_from_null.min(self.halfmove_clock() as usize);

        if end >= 4 {
            let mut idx = self.state_stack.len() as isize - 4;
            for i in (4..=end).step_by(2) {
                if idx < 0 {
                    break;
                }

                let stp = &self.state_stack[idx as usize];

                if stp.key == self.state.key {
                    self.state.repetition = if stp.repetition != 0 { -(i as i32) } else { i as i32 };
                    break;
                }

                idx -= 2;
            }
        }
    }

    pub fn undo_move(&mut self, mv: Move) {
        self.side_to_move = !self.side_to_move;

        let from = mv.from();
        let to = mv.to();
        let piece = self.piece_on(to);
        let stm = self.side_to_move;

        if !mv.is_castling() {
            self.add_piece(piece, from);
            self.remove_piece(piece, to);
        }

        if let Some(piece) = self.state.captured {
            self.add_piece(piece, to);
        }

        match mv.kind() {
            MoveKind::EnPassant => {
                self.add_piece(Piece::new(!stm, PieceType::Pawn), to ^ 8);
            }
            MoveKind::Castling => {
                let (rook_from, rook_to) = self.get_castling_rook(to);

                self.remove_piece(Piece::new(stm, PieceType::Rook), rook_to);
                self.remove_piece(piece, to);

                self.add_piece(Piece::new(stm, PieceType::Rook), rook_from);
                self.add_piece(piece, from);
            }
            _ if mv.is_promotion() => {
                self.remove_piece(piece, from);
                self.add_piece(Piece::new(stm, PieceType::Pawn), from);
            }
            _ => (),
        }

        self.state = self.state_stack.pop().unwrap();
    }
}
