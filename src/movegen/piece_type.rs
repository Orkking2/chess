use crate::bitboard::{BitBoard, EMPTY};
use crate::board::Board;
use crate::color::Color;
use crate::movegen::{MoveList, SquareAndBitBoard};
use crate::piece::Piece;
use crate::square::Square;

use crate::magic::{
    between, get_adjacent_files, get_bishop_moves, get_bishop_rays, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_moves, get_rank, get_rook_moves, get_rook_rays,
    line,
};

pub trait PieceType {
    fn into_piece() -> Piece;

    #[allow(dead_code)] //? What is the purpose of this function
    #[inline(always)]
    fn is(piece: Piece) -> bool {
        Self::into_piece() == piece
    }

    fn pseudo_legals(
        src: Square,
        color: Color,
        combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard;

    #[inline(always)]
    fn legals<const IN_CHECK: bool>(
        movelist: &mut MoveList,
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & check_mask;
            if moves != EMPTY {
                unsafe {
                    movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                }
            }
        }

        if !IN_CHECK {
            for src in pieces & pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & line(src, ksq);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn has_legals<const IN_CHECK: bool>(board: &Board, unoccupied_by_me: BitBoard) -> bool {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & check_mask;
            if moves != EMPTY {
                return true;
            }
        }

        if !IN_CHECK {
            for src in pieces & pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & line(src, ksq);
                if moves != EMPTY {
                    return true;
                }
            }
        }

        false
    }
}

pub struct PawnType;
pub struct BishopType;
pub struct KnightType;
pub struct RookType;
pub struct QueenType;
pub struct KingType;

impl PawnType {
    /// Is a particular en-passant capture legal?
    #[inline(always)]
    pub fn legal_ep_move(board: &Board, source: Square, dest: Square) -> bool {
        let combined = board.combined()
            ^ BitBoard::from_square(board.en_passant().unwrap())
            ^ BitBoard::from_square(source)
            ^ BitBoard::from_square(dest);

        let ksq = board.king_square(board.side_to_move());

        let rooks = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        if (get_rook_rays(ksq) & rooks) != EMPTY && (get_rook_moves(ksq, combined) & rooks) != EMPTY
        {
            return false;
        }

        let bishops = (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        if (get_bishop_rays(ksq) & bishops) != EMPTY
            && (get_bishop_moves(ksq, combined) & bishops) != EMPTY
        {
            return false;
        }

        true
    }
}

impl PieceType for PawnType {
    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::Pawn
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        color: Color,
        combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        get_pawn_moves(src, color, combined) & unoccupied_by_me
    }

    #[inline(always)]
    fn legals<const IN_CHECK: bool>(
        movelist: &mut MoveList,
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & check_mask;
            if moves != EMPTY {
                unsafe {
                    movelist.push_unchecked(SquareAndBitBoard::new(
                        src,
                        moves,
                        src.get_rank() == color.to_seventh_rank(),
                    ));
                }
            }
        }

        if !IN_CHECK {
            for src in pieces & pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & line(ksq, src);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            moves,
                            src.get_rank() == color.to_seventh_rank(),
                        ));
                    }
                }
            }
        }

        if board.en_passant().is_some() {
            let ep_sq = board.en_passant().unwrap();
            let rank = get_rank(ep_sq.get_rank());
            let files = get_adjacent_files(ep_sq.get_file());
            for src in rank & files & pieces {
                let dest = ep_sq.uforward(color);
                if PawnType::legal_ep_move(board, src, dest) {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            BitBoard::from_square(dest),
                            false,
                        ));
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn has_legals<const IN_CHECK: bool>(
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) -> bool {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & check_mask;
            if moves != EMPTY {
                return true;
            }
        }

        if !IN_CHECK {
            for src in pieces & pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me) & line(ksq, src);
                if moves != EMPTY {
                    return true;
                }
            }
        }

        if board.en_passant().is_some() {
            let ep_sq = board.en_passant().unwrap();
            let rank = get_rank(ep_sq.get_rank());
            let files = get_adjacent_files(ep_sq.get_file());
            for src in rank & files & pieces {
                let dest = ep_sq.uforward(color);
                if PawnType::legal_ep_move(board, src, dest) {
                    return true;
                }
            }
        }

        false
    }
}

impl PieceType for BishopType {
    #[inline(always)]
    fn is(piece: Piece) -> bool {
        piece == Piece::Bishop
    }

    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::Bishop
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        _color: Color,
        combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        get_bishop_moves(src, combined) & unoccupied_by_me
    }
}

impl PieceType for KnightType {
    #[inline(always)]
    fn is(piece: Piece) -> bool {
        piece == Piece::Knight
    }

    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::Knight
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        _color: Color,
        _combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        get_knight_moves(src) & unoccupied_by_me
    }

    #[inline(always)]
    fn legals<const IN_CHECK: bool>(
        movelist: &mut MoveList,
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        if IN_CHECK {
            let check_mask = between(checkers.to_square(), ksq) ^ checkers;

            for src in pieces & !pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me & check_mask);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        } else {
            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        };
    }

    #[inline(always)]
    fn has_legals<const IN_CHECK: bool>(
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) -> bool {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        if IN_CHECK {
            let check_mask = between(checkers.to_square(), ksq) ^ checkers;

            for src in pieces & !pinned {
                let moves =
                    Self::pseudo_legals(src, color, *combined, unoccupied_by_me & check_mask);
                if moves != EMPTY {
                    return true;
                }
            }
        } else {
            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *combined, unoccupied_by_me);
                if moves != EMPTY {
                    return true;
                }
            }
        };

        false
    }
}

impl PieceType for RookType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Rook
    }

    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::Rook
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        _color: Color,
        combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        get_rook_moves(src, combined) & unoccupied_by_me
    }
}

impl PieceType for QueenType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Queen
    }

    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::Queen
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        _color: Color,
        combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        (get_rook_moves(src, combined) ^ get_bishop_moves(src, combined)) & unoccupied_by_me
    }
}

impl KingType {
    /// Is a particular king move legal?
    #[inline(always)]
    pub fn legal_king_move(board: &Board, dest: Square) -> bool {
        let combined = board.combined()
            ^ (board.pieces(Piece::King) & board.color_combined(board.side_to_move()))
            | BitBoard::from_square(dest);

        let mut attackers = EMPTY;

        let rooks = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        attackers |= get_rook_moves(dest, combined) & rooks;

        let bishops = (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        attackers |= get_bishop_moves(dest, combined) & bishops;

        let knight_rays = get_knight_moves(dest);
        attackers |=
            knight_rays & board.pieces(Piece::Knight) & board.color_combined(!board.side_to_move());

        let king_rays = get_king_moves(dest);
        attackers |=
            king_rays & board.pieces(Piece::King) & board.color_combined(!board.side_to_move());

        attackers |= get_pawn_attacks(
            dest,
            board.side_to_move(),
            board.pieces(Piece::Pawn) & board.color_combined(!board.side_to_move()),
        );

        attackers == EMPTY
    }
}

impl PieceType for KingType {
    fn is(piece: Piece) -> bool {
        piece == Piece::King
    }

    #[inline(always)]
    fn into_piece() -> Piece {
        Piece::King
    }

    #[inline(always)]
    fn pseudo_legals(
        src: Square,
        _color: Color,
        _combined: BitBoard,
        unoccupied_by_me: BitBoard,
    ) -> BitBoard {
        get_king_moves(src) & unoccupied_by_me
    }

    #[inline(always)]
    fn legals<const IN_CHECK: bool>(
        movelist: &mut MoveList,
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) {
        let combined = board.combined();
        let color = board.side_to_move();
        let ksq = board.king_square(color);

        let mut moves = Self::pseudo_legals(ksq, color, *combined, unoccupied_by_me);

        let copy = moves;
        for dest in copy {
            if !KingType::legal_king_move(board, dest) {
                moves ^= BitBoard::from_square(dest);
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `Board` structure says we can.
        //  * the squares between my king and my rook are empty.
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !IN_CHECK {
            if board.my_castle_rights().has_kingside()
                && (combined & board.my_castle_rights().kingside_squares(color)) == EMPTY
            {
                let middle = ksq.uright();
                let right = middle.uright();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, right)
                {
                    moves ^= BitBoard::from_square(right);
                }
            }

            if board.my_castle_rights().has_queenside()
                && (combined & board.my_castle_rights().queenside_squares(color)) == EMPTY
            {
                let middle = ksq.uleft();
                let left = middle.uleft();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, left)
                {
                    moves ^= BitBoard::from_square(left);
                }
            }
        }
        if moves != EMPTY {
            unsafe {
                movelist.push_unchecked(SquareAndBitBoard::new(ksq, moves, false));
            }
        }
    }

    #[inline(always)]
    fn has_legals<const IN_CHECK: bool>(
        board: &Board,
        unoccupied_by_me: BitBoard,
    ) -> bool {
        let combined = board.combined();
        let color = board.side_to_move();
        let ksq = board.king_square(color);

        let mut moves = Self::pseudo_legals(ksq, color, *combined, unoccupied_by_me);

        let copy = moves;
        for dest in copy {
            if !KingType::legal_king_move(board, dest) {
                moves ^= BitBoard::from_square(dest);
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `Board` structure says we can.
        //  * the squares between my king and my rook are empty.
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !IN_CHECK {
            if board.my_castle_rights().has_kingside()
                && (combined & board.my_castle_rights().kingside_squares(color)) == EMPTY
            {
                let middle = ksq.uright();
                let right = middle.uright();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, right)
                {
                    moves ^= BitBoard::from_square(right);
                }
            }

            if board.my_castle_rights().has_queenside()
                && (combined & board.my_castle_rights().queenside_squares(color)) == EMPTY
            {
                let middle = ksq.uleft();
                let left = middle.uleft();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, left)
                {
                    moves ^= BitBoard::from_square(left);
                }
            }
        }

        moves != EMPTY
    }
}
