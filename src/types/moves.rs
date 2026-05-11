use super::{PieceType, Square};
use crate::board::Board;

/// Represents a chess move containing the from and to squares, as well as flags for special moves.
/// The information encoded as a 16-bit integer, 6 bits for the from/to square and 4 bits for the flags.
///
/// See [Encoding Moves](https://www.chessprogramming.org/Encoding_Moves) for more information.
#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub struct Move(u16);

/// Represents a typed enumeration of move kinds, which is the 4-bit part of the encoded bit move.
/// 
/// See [From-To Based](https://www.chessprogramming.org/Encoding_Moves#From-To_Based) for more information.
//#[rustfmt::skip]

impl Move {

    pub const DoublePush: u16        = 0b0001;
    pub const Normal: u16            = 0b0000;
    pub const Castling: u16          = 0b0010;
    pub const Capture: u16           = 0b0100;
    pub const EnPassant: u16         = 0b0101;
    pub const PromotionN: u16        = 0b1000;
    pub const PromotionB: u16        = 0b1001;
    pub const PromotionR: u16        = 0b1010;
    pub const PromotionQ: u16        = 0b1011;
    pub const PromotionCaptureN: u16 = 0b1100;
    pub const PromotionCaptureB: u16 = 0b1101;
    pub const PromotionCaptureR: u16 = 0b1110;
    pub const PromotionCaptureQ: u16 = 0b1111;

    pub const NULL: Self = Self(0);

    pub const fn new(from: Square, to: Square, kind: u16) -> Self {
        Self(from as u16 | ((to as u16) << 6) | (kind << 12))
    }

    pub const fn from(self) -> Square {
        Square::new((self.0 & 0b0011_1111) as u8)
    }

    pub const fn to(self) -> Square {
        Square::new(((self.0 >> 6) & 0b0011_1111) as u8)
    }

    pub const fn encoded(self) -> usize {
        (self.0 & 0b0000_1111_1111_1111) as usize
    }

    pub const fn kind(self) -> u16 {
        self.0 >> 12
    }

    pub const fn is_present(self) -> bool {
        !self.is_null()
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub const fn is_quiet(self) -> bool {
        self.is_present() && !self.is_noisy()
    }

    pub const fn is_noisy(self) -> bool {
        (self.0 & (7 << 12)) > (2 << 12)
    }

    pub const fn is_special(self) -> bool {
        (self.0 & 0b1011_0000_0000_0000) != 0
    }

    pub const fn is_capture(self) -> bool {
        self.0 & (1 << 14) != 0
    }

    pub const fn is_promotion(self) -> bool {
        self.0 & (1 << 15) != 0
    }

    pub const fn is_en_passant(self) -> bool {
        (self.0 >> 12) == Self::EnPassant as u16
    }

    pub fn capture_sq(self) -> Square {
        self.to() ^ (self.is_en_passant() as u8 * 8)
    }

    pub const fn is_castling(self) -> bool {
        (self.0 >> 12) == Self::Castling as u16
    }

    pub const fn is_double_push(self) -> bool {
        (self.0 >> 12) == Self::DoublePush as u16
    }

    pub const fn promo_piece_type(self) -> PieceType {
        debug_assert!(self.is_promotion());
        PieceType::new(((self.kind() as usize) & 3) + PieceType::Knight as usize)
    }

    pub fn to_uci(self, board: &Board) -> String {
        // For FRC castling moves are encoded as king capturing rook
        if board.is_frc() && self.is_castling() {
            let king_from = self.from();
            let (rook_from, _) = board.get_castling_rook(self.to());
            return format!("{king_from}{rook_from}");
        }

        let mut output = format!("{}{}", self.from(), self.to());

        if self.is_promotion() {
            match self.promo_piece_type() {
                PieceType::Knight => output.push('n'),
                PieceType::Bishop => output.push('b'),
                PieceType::Rook => output.push('r'),
                PieceType::Queen => output.push('q'),
                _ => (),
            }
        }

        output
    }
}
