use crate::ffi::{Fen, Subfen};
use aix_chess_compression::{CompressionLevel, Decode, Decoder, EncodedGame};
use shakmaty::{
    fen::{Fen as ShakmatyFen, ParseFenError},
    Board, CastlingMode, Chess, Color, FromSetup, Position,
};

const SECOND_RANK: u64 = 0x0000_0000_0000_ff00;
const SEVENTH_RANK: u64 = 0x00ff_0000_0000_0000;

pub fn try_parse(subfen: &[u8]) -> Result<Subfen, ParseFenError> {
    let board = Board::from_ascii_board_fen(subfen)?;
    Ok(subfen_from_board(&board))
}

pub fn try_parse_fen(fen: &[u8]) -> Result<Fen, ParseFenError> {
    let parsed = ShakmatyFen::from_ascii(fen)?;
    let setup = parsed.as_setup();
    let castling_rights = CastlingMode::detect(setup);
    let position: Chess = Chess::from_setup(setup.clone(), castling_rights).map_err(
        |_| ParseFenError::InvalidFen, // Map any error to InvalidFen for simplicity
    )?;
    let subfen = subfen_from_board(position.board());
    Ok(Fen {
        white: subfen.white,
        black: subfen.black,
        king: subfen.king,
        queen: subfen.queen,
        rook: subfen.rook,
        bishop: subfen.bishop,
        knight: subfen.knight,
        pawn: subfen.pawn,
        white_to_move: position.turn() == Color::White,
        castling_rights: position.castles().castling_rights().0,
        ep_square: position
            .legal_ep_square()
            .map(|square| square.to_u32() as i8)
            .unwrap_or(-1),
    })
}

fn subfen_from_board(board: &Board) -> Subfen {
    Subfen {
        white: board.white().0,
        black: board.black().0,
        king: board.kings().0,
        queen: board.queens().0,
        rook: board.rooks().0,
        bishop: board.bishops().0,
        knight: board.knights().0,
        pawn: board.pawns().0,
    }
}

pub fn matches(subfen: Subfen, game: &[u8]) -> Result<bool, crate::ffi::DecodeError> {
    matches_with_initial_fen(subfen, game, None)
}

pub fn matches_with_initial_fen(
    subfen: Subfen,
    game: &[u8],
    initial_fen: Option<&str>,
) -> Result<bool, crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(game)?;
    let decoder = Decoder::new_with_initial_fen(&encoded, initial_fen)?;
    for position in decoder.into_iter_positions() {
        let position = position?;
        let board = position.board();
        if matches_board(&subfen, board) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn matches_fen(fen: Fen, game: &[u8]) -> Result<bool, crate::ffi::DecodeError> {
    matches_fen_with_initial_fen(fen, game, None)
}

pub fn matches_fen_ply(fen: Fen, game: &[u8]) -> Result<u16, crate::ffi::DecodeError> {
    matches_fen_ply_with_initial_fen(fen, game, None)
}

pub fn matches_fen_with_initial_fen(
    fen: Fen,
    game: &[u8],
    initial_fen: Option<&str>,
) -> Result<bool, crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(game)?;
    let initial_position = initial_position_for_matching(&encoded, initial_fen)?;
    if matches_fen_state(&fen, &initial_position) {
        return Ok(true);
    }

    let mut decoder = Decoder::new_with_initial_fen(&encoded, initial_fen)?;
    let target_pawn_home = get_pawn_home_for_fen(&fen);

    loop {
        let position = decoder.next_position();
        if let Some(position) = position {
            let position = position?;
            let board = position.board();
            if !is_end_reachable(target_pawn_home, get_pawn_home(board)) {
                return Ok(false);
            }
        
            if matches_fen_state(&fen, position) {
                return Ok(true);
            }

        } else {
            break;
        }
    }

    Ok(false)
}

pub fn matches_fen_ply_with_initial_fen(
    fen: Fen,
    game: &[u8],
    initial_fen: Option<&str>,
) -> Result<u16, crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(game)?;
    let initial_position = initial_position_for_matching(&encoded, initial_fen)?;
    if matches_fen_state(&fen, &initial_position) {
        return Ok(0);
    }

    let mut decoder = Decoder::new_with_initial_fen(&encoded, initial_fen)?;
    let target_pawn_home = get_pawn_home_for_fen(&fen);
    let mut ply: u16 = 0;

    loop {
        let position = decoder.next_position();
        if let Some(position) = position {
            ply = ply.saturating_add(1);
            let position = position?;
            let board = position.board();
            if !is_end_reachable(target_pawn_home, get_pawn_home(board)) {
                return Err(crate::ffi::DecodeError::NoErrorNoValue);
            }

            if matches_fen_state(&fen, position) {
                return Ok(ply);
            }
        } else {
            break;
        }
    }

    Err(crate::ffi::DecodeError::NoErrorNoValue)
}

fn initial_position_for_matching(
    encoded: &EncodedGame,
    initial_fen: Option<&str>,
) -> Result<Chess, crate::ffi::DecodeError> {
    if encoded.compression_level() != CompressionLevel::Low {
        return Ok(Chess::new());
    }

    let Some(initial_fen) = initial_fen else {
        return Ok(Chess::new());
    };

    let parsed = ShakmatyFen::from_ascii(initial_fen.as_bytes())
        .map_err(|_| crate::ffi::DecodeError::InvalidDataDuringDecoding)?;
    let setup = parsed.as_setup();
    let castling_mode = CastlingMode::detect(setup);
    Chess::from_setup(setup.clone(), castling_mode)
        .map_err(|_| crate::ffi::DecodeError::InvalidDataDuringDecoding)
}

fn get_pawn_home(board: &Board) -> u16 {
    let white_pawns = (board.pawns() & board.white()).0;
    let black_pawns = (board.pawns() & board.black()).0;
    let second_rank_pawns = ((white_pawns & SECOND_RANK) >> 8) as u8;
    let seventh_rank_pawns = ((black_pawns & SEVENTH_RANK) >> 48) as u8;
    (second_rank_pawns as u16) | ((seventh_rank_pawns as u16) << 8)
}

fn get_pawn_home_for_fen(fen: &Fen) -> u16 {
    let white_pawns = fen.pawn & fen.white;
    let black_pawns = fen.pawn & fen.black;
    let second_rank_pawns = ((white_pawns & SECOND_RANK) >> 8) as u8;
    let seventh_rank_pawns = ((black_pawns & SEVENTH_RANK) >> 48) as u8;
    (second_rank_pawns as u16) | ((seventh_rank_pawns as u16) << 8)
}

/// Returns true if the end pawn structure is reachable.
fn is_end_reachable(end: u16, pos: u16) -> bool {
    end & !pos == 0
}

pub fn matches_board(subfen: &Subfen, board: &Board) -> bool {
    (board.white().0 & subfen.white) == subfen.white
        && (board.black().0 & subfen.black) == subfen.black
        && (board.kings().0 & subfen.king) == subfen.king
        && (board.queens().0 & subfen.queen) == subfen.queen
        && (board.rooks().0 & subfen.rook) == subfen.rook
        && (board.bishops().0 & subfen.bishop) == subfen.bishop
        && (board.knights().0 & subfen.knight) == subfen.knight
        && (board.pawns().0 & subfen.pawn) == subfen.pawn
}

fn matches_fen_state(fen: &Fen, position: &Chess) -> bool {
    let board = position.board();
    board.white().0 == fen.white
        && board.black().0 == fen.black
        && board.kings().0 == fen.king
        && board.queens().0 == fen.queen
        && board.rooks().0 == fen.rook
        && board.bishops().0 == fen.bishop
        && board.knights().0 == fen.knight
        && board.pawns().0 == fen.pawn
        && (position.turn() == Color::White) == fen.white_to_move
        && position.castles().castling_rights().0 == fen.castling_rights
        && position
            .legal_ep_square()
            .map(|square| square.to_u32() as i8)
            .unwrap_or(-1)
            == fen.ep_square
}

/// Finds the first occurrence and returns its next move in one decoding pass.
/// Zero means the game ended at the target. Otherwise the value is one plus
/// from | (to << 6) | (promotion ASCII << 12). NoErrorNoValue means no match.
pub fn next_move_at_fen(
    fen: &Fen,
    game: &[u8],
    initial_fen: Option<&str>,
) -> Result<u32, crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(game)?;
    let target_home = get_pawn_home_for_fen(fen);
    let target_count = (fen.white | fen.black).count_ones() as usize;
    // Reject impossible home-pawn and capture histories before reconstructing boards.
    // Custom starts and other encodings retain the general decoder path.
    if initial_fen.is_none()
        && encoded.compression_level() == CompressionLevel::Low
        && home_pawns_exclude(game, target_home, target_count)
    {
        return Err(crate::ffi::DecodeError::NoErrorNoValue);
    }

    let initial = initial_position_for_matching(&encoded, initial_fen)?;
    let mut decoder = Decoder::new_with_initial_fen(&encoded, initial_fen)?;
    if !matches_fen_state(fen, &initial) {
        loop {
            let Some(position) = decoder.next_position() else {
                return Err(crate::ffi::DecodeError::NoErrorNoValue);
            };
            let position = position?;
            if position.board().occupied().count() < target_count
                || !is_end_reachable(target_home, get_pawn_home(position.board())) {
                return Err(crate::ffi::DecodeError::NoErrorNoValue);
            }
            if matches_fen_state(fen, position) {
                break;
            }
        }
    }

    let Some(m) = decoder.next_move() else { return Ok(0); };
    let m = m?;
    let to = if let shakmaty::Move::Castle { king, rook } = m {
        let side = shakmaty::CastlingSide::from_king_side(king < rook);
        shakmaty::Square::from_coords(side.king_to_file(), king.rank())
    } else { m.to() };
    let from = m.from().ok_or(crate::ffi::DecodeError::InvalidDataDuringDecoding)?;
    let promotion = m.promotion().map(|p| p.char() as u32).unwrap_or(0);
    Ok(1 + (from as u32 | ((to as u32) << 6) | (promotion << 12)))
}

/// Low encoding stores source/destination in two-byte records. Home pawns
/// cannot return, so departures/captures can rule out a target without boards.
/// Captures reduce total piece count; promotions preserve it. Stop at the first
/// compatible mask and piece count; never follow variation records.
fn home_pawns_exclude(game: &[u8], target: u16, target_count: usize) -> bool {
    let mut home = u16::MAX;
    let mut count = 32usize;
    if (target == home && target_count == count) || game.len() % 2 != 1 { return false; }
    let mut depth = 0u32;
    for pair in game[..game.len() - 1].chunks_exact(2) {
        match pair {
            [255, 255] => { depth += 1; continue; }
            [255, 254] => {
                if depth == 0 { return false; }
                depth -= 1;
                continue;
            }
            [255, _] => return false,
            _ => {}
        }
        if depth != 0 { continue; }
        let from = pair[0] & 63;
        let captured = if pair[0] & 128 != 0 { Some(pair[1] & 63) } else { None };
        if captured.is_some() {
            if count == 0 { return false; }
            count -= 1;
        }
        if count < target_count { return true; }
        for sq in [Some(from), captured].into_iter().flatten() {
            if (8..16).contains(&sq) { home &= !(1 << (sq - 8)); }
            if (48..56).contains(&sq) { home &= !(1 << (sq - 40)); }
        }
        if !is_end_reachable(target, home) { return true; }
        if home == target && count == target_count { return false; }
    }
    // No mainline position had both the required home pawns and piece count.
    depth == 0
}

#[cfg(test)]
mod next_move_tests {
    use super::*;
    use aix_chess_compression::{Encode, Encoder};

    #[test]
    fn capture_prefilter_requires_compatible_count_on_mainline() {
        let home = u16::MAX;
        assert!(home_pawns_exclude(&[0], home, 31));
        assert!(!home_pawns_exclude(&[24 | 128, 32, 0], home, 31));
        assert!(home_pawns_exclude(&[24 | 128, 32, 0], home, 30));
        // Captures inside nested variations must not reduce mainline material.
        assert!(home_pawns_exclude(&[255, 255, 255, 255, 152, 32, 255, 254, 255, 254, 0], home, 31));
        // Promotion alone preserves total material; capture-promotion loses one.
        let missing_a7 = home & !(1 << 8);
        assert!(home_pawns_exclude(&[48 | 64, 56, 0], missing_a7, 31));
        assert!(!home_pawns_exclude(&[48 | 64 | 128, 57, 0], missing_a7, 31));
    }

    #[test]
    fn zero_home_prefilter_tracks_mainline_and_falls_back_on_bad_markers() {
        assert!(home_pawns_exclude(&[0], 0, 32));
        assert!(home_pawns_exclude(&[12, 28, 0], 0, 32));
        let mut departed = Vec::new();
        for sq in (8..16).chain(48..56) {
            departed.extend_from_slice(&[sq, 24]);
        }
        departed.push(0);
        assert!(!home_pawns_exclude(&departed, 0, 32));
        // Captures on a home square also remove that pawn from the mask.
        departed[30] = 24 | 128;
        departed[31] = 55;
        assert!(!home_pawns_exclude(&departed, 0, 31));
        let mut variation = vec![255, 255, 255, 255];
        variation.extend_from_slice(&departed[..departed.len() - 1]);
        variation.extend_from_slice(&[255, 254, 255, 254, 0]);
        assert!(home_pawns_exclude(&variation, 0, 31));
        for malformed in [vec![255, 255, 0], vec![255, 254, 0], vec![255, 1, 0], vec![12, 0]] {
            assert!(!home_pawns_exclude(&malformed, 0, 32));
        }
    }

    fn check(target: &str, bytes: &[u8], initial: Option<&str>, expected: Option<u32>) {
        let fen = try_parse_fen(target.as_bytes()).unwrap();
        let actual = match next_move_at_fen(&fen, bytes, initial) {
            Ok(m) => Some(m),
            Err(crate::ffi::DecodeError::NoErrorNoValue) => None,
            Err(_) => panic!("unexpected decode error"),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn special_moves_and_terminal_positions() {
        let initial = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let e4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        check(initial, &[12, 28, 0], None, Some(1 + 12 + (28 << 6)));
        check(e4, &[12, 28, 0], None, Some(0));
        check(e4, &[11, 27, 0], None, None);
        check(e4, &[12, 28, 255, 255, 11, 27, 255, 254, 52, 36, 0], None, Some(1 + 52 + (36 << 6)));
        let castle = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";
        check(castle, &[4, 7, 0], Some(castle), Some(1 + 4 + (6 << 6)));
        let promotion = "7k/P7/8/8/8/8/8/7K w - - 0 1";
        check(promotion, &[48 | 64, 56, 0], Some(promotion), Some(1 + 48 + (56 << 6) + (('q' as u32) << 12)));
        let ep = "7k/8/8/3pP3/8/8/8/7K w - d6 0 1";
        check(ep, &[36 | 128, 43, 0], Some(ep), Some(1 + 36 + (43 << 6)));
    }

    #[test]
    fn agrees_with_existing_search_across_encodings() {
        // Deterministic legal games exercise captures, castling rights and
        // arbitrary targets in all three formats, including absent positions.
        for seed in 0..12usize {
            let mut pos = Chess::new();
            let mut positions = vec![pos.clone()];
            let mut moves = Vec::new();
            for ply in 0..100usize {
                let legal = pos.legal_moves();
                if legal.is_empty() { break; }
                let m = legal[(seed * 31 + ply * 17) % legal.len()];
                pos.play_unchecked(m);
                moves.push(m);
                positions.push(pos.clone());
            }
            for level in [CompressionLevel::Low, CompressionLevel::Medium, CompressionLevel::High] {
                let mut encoder = Encoder::new(level);
                for m in &moves { encoder.encode_move(*m).ok().unwrap(); }
                let bytes = encoder.finish().into_bytes();
                for pos in positions.iter().step_by(7) {
                    let text = ShakmatyFen::from_position(pos, shakmaty::EnPassantMode::Legal).to_string();
                    let target = try_parse_fen(text.as_bytes()).unwrap();
                    let ply = matches_fen_ply(try_parse_fen(text.as_bytes()).unwrap(), &bytes).ok().unwrap();
                    let expected = crate::game::from_bytes(&bytes).ok().unwrap();
                    let expected = expected.move_details_iterator().0.nth(ply as usize).map(|m| m.ok().unwrap());
                    let expected = expected.map(|m| 1 + m.from as u32 + ((m.to as u32) << 6) + ((m.promotion as u32) << 12)).unwrap_or(0);
                    assert_eq!(next_move_at_fen(&target, &bytes, None).ok(), Some(expected));
                }
            }
        }
    }
}
