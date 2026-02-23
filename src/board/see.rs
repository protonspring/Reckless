use crate::{
            //(self.pieces(attacker) & our_attackers).lsb());
    lookup::{between, ray_pass, bishop_attacks, rook_attacks},
    types::{Bitboard, Color, Move, PieceType},
};

impl super::Board {
    /// Checks if the static exchange evaluation (SEE) of a move meets the given `threshold`,
    /// indicating that the sequence of captures on a single square, starting with the move,
    /// results in a value greater than or equal to the threshold for the side to move.
    ///
    /// Promotions and castling always pass this check.
    pub fn see(&self, mv: Move, threshold: i32) -> bool {
        if mv.is_castling() {
            return true;
        }

        // In the best case, we win a piece, but still end up with a negative balance
        let mut balance = self.move_value(mv) - threshold;
        if balance < 0 {
            return false;
        }

        // In the worst case, we lose a piece, but still end up with a non-negative balance
        balance -= self.piece_on(mv.from()).value();

        if let Some(promotion) = mv.promotion_piece() {
            balance -= promotion.value();
        }

        if balance >= 0 {
            return true;
        }

        let mut occupancies = self.occupancies();
        occupancies.clear(mv.from());
        occupancies.set(mv.to());

        if mv.is_en_passant() {
            occupancies.clear(mv.to() ^ 8);
        }

        let mut attackers = self.attackers_to(mv.to(), occupancies) & occupancies;
        let mut stm = !self.side_to_move();

        let diagonal = self.pieces(PieceType::Bishop) | self.pieces(PieceType::Queen);
        let orthogonal = self.pieces(PieceType::Rook) | self.pieces(PieceType::Queen);

        let white_pins = self.pinned(Color::White) & !between(self.king_square(Color::White), mv.to());
        let black_pins = self.pinned(Color::Black) & !between(self.king_square(Color::Black), mv.to());

        let white_pinner = self.pinner(Color::White) & !ray_pass(self.king_square(Color::Black), mv.to());
        let black_pinner = self.pinner(Color::Black) & !ray_pass(self.king_square(Color::White), mv.to());

        let mut allowed = !(white_pins | black_pins);

        let unaligned_pinners = white_pinner | black_pinner;

        //println!("{self}");
        //println!("Move: {} - {}", mv.from(), mv.to());

        loop {

            // Allow all pieces on this stm, if the enemy pinners are gone
            //if (occupancies & self.pinner(!stm)).is_empty() {
                //allowed = allowed | self.colors(stm);
            //}

            let our_attackers = attackers & allowed & self.colors(stm);
            if our_attackers.is_empty() {
                break;
            }

            let attacker = self.least_valuable_attacker(our_attackers);

            // The king cannot capture a protected piece; the side to move loses the exchange
            if attacker == PieceType::King && !(attackers & self.colors(!stm)).is_empty() {
                break;
            }

            // Make the capture
            let the_attacker = (self.pieces(attacker) & our_attackers).lsb();

            //println!("Attack from: {}", the_attacker);

            //if (self.pinner(stm) & the_attacker.to_bb() & (white_pins | black_pins)) != Bitboard(0) { 
            if (the_attacker.to_bb() & unaligned_pinners) != Bitboard(0) { 
                //println!("unaligned pinner gone");
                allowed |= between(self.king_square(!stm), the_attacker);
            }
            //if (the_attacker.to_bb() & self.pinned(stm) & (white_pins | black_pins)) != Bitboard(0) { 
                //println!("previously not allowed allowed");
            //}

            occupancies.clear(the_attacker);
            stm = !stm;

            // Assume our piece is going to be captured
            balance = -balance - 1 - attacker.value();
            if balance >= 0 {
                break;
            }

            // Capturing a piece may reveal a new sliding attacker
            if [PieceType::Pawn, PieceType::Bishop, PieceType::Queen].contains(&attacker) {
                attackers |= bishop_attacks(mv.to(), occupancies) & diagonal;
            }
            if [PieceType::Rook, PieceType::Queen].contains(&attacker) {
                attackers |= rook_attacks(mv.to(), occupancies) & orthogonal;
            }
            attackers &= occupancies;
        }

        // The last side to move has failed to capture back
        // since it has no more attackers and, therefore, is losing
        stm != self.side_to_move()
    }

    fn move_value(&self, mv: Move) -> i32 {
        if mv.is_en_passant() {
            return PieceType::Pawn.value();
        }

        let capture = self.piece_on(mv.to()).piece_type();

        if let Some(promotion) = mv.promotion_piece() {
            capture.value() + promotion.value() - PieceType::Pawn.value()
        } else {
            capture.value()
        }
    }

    fn least_valuable_attacker(&self, attackers: Bitboard) -> PieceType {
        for index in 0..PieceType::NUM {
            let piece = PieceType::new(index);
            if !(self.pieces(piece) & attackers).is_empty() {
                return piece;
            }
        }
        unreachable!();
    }
}
