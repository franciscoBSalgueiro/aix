use aix_chess_compression::CompressionLevel;

mod game;
mod scoutfish;
mod subfen;

const LEVELS: [CompressionLevel; 3] = [
    CompressionLevel::Low,
    CompressionLevel::Medium,
    CompressionLevel::High,
];

impl From<aix_chess_compression::DecodeError> for ffi::DecodeError {
    fn from(_: aix_chess_compression::DecodeError) -> Self {
        ffi::DecodeError::InvalidDataDuringDecoding
    }
}

impl From<aix_chess_compression::EncodedGameConstructionError> for ffi::DecodeError {
    fn from(e: aix_chess_compression::EncodedGameConstructionError) -> Self {
        match e {
            aix_chess_compression::EncodedGameConstructionError::EmptyData => {
                ffi::DecodeError::EmptyBlob
            }
            aix_chess_compression::EncodedGameConstructionError::InvalidCompressionLevel => {
                ffi::DecodeError::InvalidCompressionLevel
            }
            aix_chess_compression::EncodedGameConstructionError::InvalidData => {
                ffi::DecodeError::InvalidEncodedGameConstructionData
            }
        }
    }
}

fn optional_result_to_result<T>(
    option: Option<Result<T, ffi::DecodeError>>,
) -> Result<T, ffi::DecodeError> {
    match option {
        Some(Ok(value)) => Ok(value),
        Some(Err(e)) => Err(e),
        None => Err(ffi::DecodeError::NoErrorNoValue),
    }
}

#[diplomat::bridge]
mod ffi {
    use std::io::Write;

    use aix_chess_compression::EncodedGame;

    #[derive(Debug)]
    pub enum ScoutfishQueryParseError {
        InvalidPiece = 1,
        InvalidImbalanceFormat = 2,
        InvalidMaterialFormat = 3,
        InvalidSideToMove = 4,
        InvalidSan = 5,
        InvalidSyntaxOrStructure = 6,
        BincodeError = 7,
        BufferTooSmall = 8,
        CursorWriteError = 9,
    }

    pub struct Bitboards {
        pub w_k: u64,
        pub w_q: u64,
        pub w_r: u64,
        pub w_b: u64,
        pub w_n: u64,
        pub w_p: u64,
        pub b_k: u64,
        pub b_q: u64,
        pub b_r: u64,
        pub b_b: u64,
        pub b_n: u64,
        pub b_p: u64,
    }

    pub struct MoveDetailsExtended {
        pub ply: u16,
        pub role: i8,
        pub from: u8,
        pub to: u8,
        pub capture: i8,
        pub is_castle: bool,
        pub promotion: i8,
        pub is_en_passant: bool,
        pub is_check: bool,
        pub is_checkmate: bool,
        pub is_stalemate: bool,
        pub legal_response_move_count: u8,
    }

    pub struct MoveDetails {
        pub ply: u16,
        pub role: i8,
        pub from: u8,
        pub to: u8,
        pub capture: i8,
        pub is_castle: bool,
        pub promotion: i8,
        pub is_en_passant: bool,
        pub is_check: bool,
        pub is_checkmate: bool,
    }

    pub enum DecodeError {
        NoErrorNoValue = 0,
        EmptyBlob = 1,
        InvalidCompressionLevel = 2,
        InvalidEncodedGameConstructionData = 3,
        InvalidDataDuringDecoding = 4,
    }

    #[diplomat::opaque]
    pub struct Game<'a>(pub EncodedGame<'a>);

    #[diplomat::opaque]
    pub struct MoveDetailsExtIterator<'a>(
        pub Box<dyn Iterator<Item = Result<MoveDetailsExtended, DecodeError>> + 'a>,
    );

    #[diplomat::opaque]
    pub struct MoveDetailsIterator<'a>(
        pub Box<dyn Iterator<Item = Result<MoveDetails, DecodeError>> + 'a>,
    );

    impl<'a> Game<'a> {
        pub fn from_bytes(data: &'a [u8]) -> Result<Box<Self>, DecodeError> {
            crate::game::from_bytes(data)
        }
        pub fn pieces_at_position(data: &[u8], pos: i32) -> Result<Bitboards, DecodeError> {
            crate::game::pieces_at_position(data, pos)
        }
        pub fn board_at_position(data: &[u8], pos: i32, out: &mut [i8]) -> Result<(), DecodeError> {
            crate::game::board_at_position(data, pos, out)
        }
        pub fn fen_at_position(
            data: &[u8],
            pos: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), DecodeError> {
            crate::game::fen_at_position(data, pos, out)
        }
        pub fn to_uci_string(data: &[u8], out: &mut DiplomatWrite) -> Result<(), DecodeError> {
            crate::game::to_uci_string(data, out)
        }
        pub fn to_pgn_string(data: &[u8], out: &mut DiplomatWrite) -> Result<(), DecodeError> {
            crate::game::to_pgn_string(data, out)
        }
        pub fn moved_pieces(data: &[u8], out: &mut DiplomatWrite) -> Result<(), DecodeError> {
            crate::game::moved_pieces(data, out)
        }

        pub fn recompress(data: &[u8], level: u8, out: &mut [u8]) -> Result<usize, DecodeError> {
            let game = EncodedGame::from_bytes(data)?;
            let recomp = game.recompress(crate::LEVELS[level as usize])?;
            let bytes = recomp.into_bytes();
            let mut cursor = std::io::Cursor::new(out);
            let written = cursor.write(&bytes).unwrap();
            assert_eq!(written, bytes.len());
            Ok(written)
        }

        pub fn move_details_ext_iterator(&'a self) -> Box<MoveDetailsExtIterator<'a>> {
            Box::new(MoveDetailsExtIterator::<'a>(Box::new(
                crate::game::move_details_ext_iterator(&self.0),
            )))
        }

        pub fn move_details_iterator(&'a self) -> Box<MoveDetailsIterator<'a>> {
            Box::new(MoveDetailsIterator::<'a>(Box::new(
                crate::game::move_details_iterator(&self.0),
            )))
        }

        pub fn is_valid_movedata(data: &[u8]) -> bool {
            crate::game::is_valid_movedata(data)
        }
    }

    impl<'a> MoveDetailsExtIterator<'a> {
        pub fn next(&mut self) -> Result<MoveDetailsExtended, DecodeError> {
            crate::optional_result_to_result(self.0.next())
        }

        pub fn nth(&mut self, n: i16) -> Result<MoveDetailsExtended, DecodeError> {
            if n >= 0 {
                crate::optional_result_to_result(self.0.nth(n as usize))
            } else {
                let mut collected = self
                    .0
                    .by_ref()
                    .collect::<Result<Vec<MoveDetailsExtended>, DecodeError>>()?;
                let i = collected.len() as i16 + n;
                if i >= 0 {
                    let result = collected.swap_remove(i as usize);
                    Ok(result)
                } else {
                    Err(DecodeError::NoErrorNoValue)
                }
            }
        }
    }

    impl<'a> MoveDetailsIterator<'a> {
        pub fn next(&mut self) -> Result<MoveDetails, DecodeError> {
            crate::optional_result_to_result(self.0.next())
        }

        pub fn nth(&mut self, n: i16) -> Result<MoveDetails, DecodeError> {
            if n >= 0 {
                crate::optional_result_to_result(self.0.nth(n as usize))
            } else {
                let mut collected = self
                    .0
                    .by_ref()
                    .collect::<Result<Vec<MoveDetails>, DecodeError>>()?;
                let i = collected.len() as i16 + n;
                if i >= 0 {
                    let result = collected.swap_remove(i as usize);
                    Ok(result)
                } else {
                    Err(DecodeError::NoErrorNoValue)
                }
            }
        }
    }

    #[cfg_attr(test, derive(Debug, PartialEq))]
    #[derive(bincode::Encode, bincode::Decode)]
    pub struct Subfen {
        pub white: u64,
        pub black: u64,
        pub king: u64,
        pub queen: u64,
        pub rook: u64,
        pub bishop: u64,
        pub knight: u64,
        pub pawn: u64,
    }

    impl Subfen {
        pub fn parse(subfen: &DiplomatStr) -> Result<Subfen, ()> {
            crate::subfen::try_parse(subfen).map_err(|_| ())
        }

        pub fn matches(self, game: &[u8]) -> Result<bool, DecodeError> {
            crate::subfen::matches(self, game)
        }
    }

    pub struct Fen {
        pub white: u64,
        pub black: u64,
        pub king: u64,
        pub queen: u64,
        pub rook: u64,
        pub bishop: u64,
        pub knight: u64,
        pub pawn: u64,
        pub white_to_move: bool,
        pub castling_rights: u64,
        pub ep_square: i8,
    }

    impl Fen {
        pub fn parse(fen: &DiplomatStr) -> Result<Fen, ()> {
            crate::subfen::try_parse_fen(fen).map_err(|_| ())
        }

        pub fn matches_fen(self, game: &[u8]) -> Result<bool, DecodeError> {
            crate::subfen::matches_fen(self, game)
        }
    }

    #[diplomat::opaque]
    pub struct ScoutfishQuery(pub crate::scoutfish::Query);

    impl ScoutfishQuery {
        pub fn parse_into_bytes(
            s: &DiplomatStr,
            out: &mut [u8],
        ) -> Result<usize, ScoutfishQueryParseError> {
            crate::scoutfish::Query::parse_into_bytes(s, out)
        }

        pub fn decode_bytes(data: &[u8]) -> Result<Box<ScoutfishQuery>, ()> {
            crate::scoutfish::Query::decode_bytes(data).map(|q| Box::new(ScoutfishQuery(q)))
        }

        pub fn matches(&self, game: &[u8]) -> Result<bool, DecodeError> {
            let game = EncodedGame::from_bytes(game)?;
            Ok(self.0.apply(&game, false)?.0)
        }

        pub fn matches_plies(&self, game: &[u8], out: &mut [u32]) -> Result<u32, DecodeError> {
            assert_eq!(out.len(), 16);
            let game = EncodedGame::from_bytes(game)?;
            if let Some(plies) = self.0.apply(&game, true)?.1 {
                let len = plies.len() as u16;
                let min = (plies[0] / 32) * 32;

                for ply in plies {
                    let ply = std::cmp::min(ply - min, 511);
                    let index = (ply / 32) as usize;
                    let bit = ply % 32;
                    out[index] |= 1 << bit;
                }

                Ok((len as u32) | ((min as u32) << 16))
            } else {
                Ok(0)
            }
        }
    }
}
