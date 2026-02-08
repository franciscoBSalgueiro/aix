use crate::ffi::{Bitboards, Game, MoveDetails};
use aix_chess_compression::{Decode, Decoder, EncodedGame};
use diplomat_runtime::DiplomatWrite;
use shakmaty::fen::Fen;
use shakmaty::{Board, Chess, Color, EnPassantMode, Position};
use std::fmt::Write;

fn board_into_bitboards(board: &Board) -> Bitboards {
    let white = board.white();
    let black = board.black();
    let kings = board.kings();
    let queens = board.queens();
    let rooks = board.rooks();
    let bishops = board.bishops();
    let knights = board.knights();
    let pawns = board.pawns();
    Bitboards {
        w_k: (white & kings).0,
        w_q: (white & queens).0,
        w_r: (white & rooks).0,
        w_b: (white & bishops).0,
        w_n: (white & knights).0,
        w_p: (white & pawns).0,
        b_k: (black & kings).0,
        b_q: (black & queens).0,
        b_r: (black & rooks).0,
        b_b: (black & bishops).0,
        b_n: (black & knights).0,
        b_p: (black & pawns).0,
    }
}

fn chess_at_position(data: &[u8], pos: i32) -> Result<Option<Chess>, crate::ffi::DecodeError> {
    let game = EncodedGame::from_bytes(data)?;

    if pos == 0 {
        return Ok(Some(Chess::new()));
    }

    let decoder = Decoder::new(&game);
    let mut pos_iter = decoder.into_iter_positions();

    if pos > 0 {
        pos_iter
            .nth((pos - 1) as usize)
            .transpose()
            .map_err(|e| e.into())
    } else {
        let mut positions = pos_iter.collect::<Result<Vec<_>, _>>()?;
        let index = positions.len() as i32 + pos;
        if index < 0 {
            Ok(None)
        } else {
            Ok(Some(positions.swap_remove(index as usize)))
        }
    }
}

pub fn pieces_at_position(data: &[u8], pos: i32) -> Result<Bitboards, crate::ffi::DecodeError> {
    let maybe_chess = chess_at_position(data, pos)?;
    if let Some(chess) = maybe_chess {
        Ok(board_into_bitboards(&chess.board()))
    } else {
        Err(crate::ffi::DecodeError::NoErrorNoValue)
    }
}

pub fn board_at_position(
    data: &[u8],
    pos: i32,
    out: &mut [i8],
) -> Result<(), crate::ffi::DecodeError> {
    let maybe_chess = chess_at_position(data, pos)?;
    if let Some(chess) = maybe_chess {
        let setup = chess.to_setup(EnPassantMode::Always);
        for (sq, p) in setup.board {
            out[sq as usize] = p.char() as i8;
        }
        Ok(())
    } else {
        return Err(crate::ffi::DecodeError::NoErrorNoValue);
    }
}

pub fn fen_at_position(
    data: &[u8],
    pos: i32,
    out: &mut DiplomatWrite,
) -> Result<(), crate::ffi::DecodeError> {
    let maybe_chess = chess_at_position(data, pos)?;
    if let Some(chess) = maybe_chess {
        let fen = Fen::from_position(&chess, EnPassantMode::Always);
        write!(out, "{fen}").expect("fen_at_position: write to DiplomatWrite failed");
        Ok(())
    } else {
        return Err(crate::ffi::DecodeError::NoErrorNoValue);
    }
}

pub fn to_uci_string(data: &[u8], out: &mut DiplomatWrite) -> Result<(), crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(data)?;
    let decoder = Decoder::new(&encoded);
    let uci_string = decoder.into_uci_string()?;
    write!(out, "{uci_string}").unwrap();
    Ok(())
}

pub fn to_pgn_string(data: &[u8], out: &mut DiplomatWrite) -> Result<(), crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(data)?;
    let decoder = Decoder::new(&encoded);
    let pgn_string = decoder.into_pgn_string()?;
    write!(out, "{pgn_string}").unwrap();
    Ok(())
}

pub fn moved_pieces(data: &[u8], out: &mut DiplomatWrite) -> Result<(), crate::ffi::DecodeError> {
    let encoded = EncodedGame::from_bytes(data)?;
    let decoder = Decoder::new(&encoded);
    for (i, m) in decoder.into_iter_moves().enumerate() {
        let piece = m?
            .role()
            .of(if i % 2 == 0 {
                Color::White
            } else {
                Color::Black
            })
            .char();
        write!(out, "{piece}").unwrap();
    }

    Ok(())
}

pub fn is_valid_movedata(data: &[u8]) -> bool {
    match EncodedGame::from_bytes(data) {
        Ok(encoded_game) => {
            let decoder = Decoder::new(&encoded_game);
            decoder.into_iter_moves().all(|m| m.is_ok())
        }
        Err(_) => false,
    }
}

pub fn from_bytes(data: &'_ [u8]) -> Result<Box<Game<'_>>, crate::ffi::DecodeError> {
    Ok(Box::new(Game(EncodedGame::from_bytes(data)?)))
}

fn castling_king_dest(king: shakmaty::Square, rook: shakmaty::Square) -> shakmaty::Square {
    let side = shakmaty::CastlingSide::from_king_side(king < rook);
    shakmaty::Square::from_coords(side.king_to_file(), king.rank())
}

pub fn move_details_iterator<'a>(
    encoded: &'a EncodedGame,
) -> impl Iterator<Item = Result<MoveDetails, crate::ffi::DecodeError>> + 'a {
    let decoder = Decoder::new(encoded);
    decoder
        .into_iter_moves_and_positions()
        .enumerate()
        .map(|(ply, r)| {
            r.map(|(m, pos)| {
                let from = m.from().expect("from() should always be Some(...)") as u8;
                let to = match m {
                    shakmaty::Move::Normal { to, .. }
                    | shakmaty::Move::EnPassant { to, .. }
                    | shakmaty::Move::Put { to, .. } => to,
                    shakmaty::Move::Castle { king, rook } => castling_king_dest(king, rook),
                } as u8;
                let capture = match m.capture() {
                    Some(role) => role.char() as i8,
                    None => 0,
                };
                let is_castle = m.is_castle();
                let promotion = match m.promotion() {
                    Some(role) => role.char() as i8,
                    None => 0,
                };
                let role = m.role().char() as i8;
                let ply = ply as u16;

                let is_check = pos.is_check();
                let is_checkmate = pos.is_checkmate();

                let is_en_passant = m.is_en_passant();

                MoveDetails {
                    ply,
                    role,
                    from,
                    to,
                    capture,
                    is_castle,
                    promotion,
                    is_check,
                    is_checkmate,
                    is_en_passant,
                }
            })
            .map_err(|e| e.into())
        })
}
