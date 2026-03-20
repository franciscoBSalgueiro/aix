use crate::ffi::Subfen;
use aix_chess_compression::{Decode, Decoder, EncodedGame};
use shakmaty::{fen::ParseFenError, Board, Position};

const SECOND_RANK: u64 = 0x0000_0000_0000_ff00;
const SEVENTH_RANK: u64 = 0x00ff_0000_0000_0000;

pub fn try_parse(subfen: &[u8]) -> Result<Subfen, ParseFenError> {
    let board = Board::from_ascii_board_fen(subfen)?;
    Ok(Subfen {
        white: board.white().0,
        black: board.black().0,
        king: board.kings().0,
        queen: board.queens().0,
        rook: board.rooks().0,
        bishop: board.bishops().0,
        knight: board.knights().0,
        pawn: board.pawns().0,
    })
}

pub fn matches(subfen: Subfen, game: &[u8]) -> Result<bool, crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(game)?;
    let decoder = Decoder::new(&encoded);
    let target_pawn_home = get_pawn_home_for_subfen(&subfen);

    for position in decoder.into_iter_positions() {
        let position = position?;
        let board = position.board();

        if !is_end_reachable(target_pawn_home, get_pawn_home(board)) {
            return Ok(false);
        }

        if matches_board(&subfen, board) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_pawn_home(board: &Board) -> u16 {
    let white_pawns = (board.pawns() & board.white()).0;
    let black_pawns = (board.pawns() & board.black()).0;
    let second_rank_pawns = ((white_pawns & SECOND_RANK) >> 8) as u8;
    let seventh_rank_pawns = ((black_pawns & SEVENTH_RANK) >> 48) as u8;
    (second_rank_pawns as u16) | ((seventh_rank_pawns as u16) << 8)
}

fn get_pawn_home_for_subfen(subfen: &Subfen) -> u16 {
    let white_pawns = subfen.pawn & subfen.white;
    let black_pawns = subfen.pawn & subfen.black;
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
