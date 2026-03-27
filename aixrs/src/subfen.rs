use crate::ffi::{Fen, Subfen};
use aix_chess_compression::{Decode, Decoder, EncodedGame};
use shakmaty::{
    Board, CastlingMode, Chess, Color, FromSetup, Position, fen::{Fen as ShakmatyFen, ParseFenError}
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
    let encoded = EncodedGame::from_bytes(game)?;
    let decoder = Decoder::new(&encoded);
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
    let encoded = EncodedGame::from_bytes(game)?;
    let decoder = Decoder::new(&encoded);
    let target_pawn_home = get_pawn_home_for_fen(&fen);

    for position in decoder.into_iter_positions() {
        let position = position?;
        let board = position.board();
        if !is_end_reachable(target_pawn_home, get_pawn_home(board)) {
            return Ok(false);
        }

        if matches_fen_state(&fen, &position) {
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
