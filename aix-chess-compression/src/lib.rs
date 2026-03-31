use chess_huffman::EncodedGame as BitsEncodedGame;
use shakmaty::{
    CastlingMode, Chess, FromSetup, Move, Position,
    fen::Fen as ShakmatyFen,
    san::{San, SanPlus, Suffix},
    uci::UciMove,
};
use std::{
    borrow::Cow,
    fmt::{self},
};

mod compactindex;
mod huffman;
mod naive;

use compactindex::{CompactIndexDecoder, CompactIndexEncoder};
use huffman::{HuffDecoder, HuffEncoder};
use naive::{NaiveDecoder, NaiveEncoder};

fn parse_initial_position(initial_fen: Option<&str>) -> Result<Option<Chess>, ()> {
    let Some(initial_fen) = initial_fen else {
        return Ok(None);
    };

    let parsed = ShakmatyFen::from_ascii(initial_fen.as_bytes()).map_err(|_| ())?;
    let setup = parsed.as_setup();
    let castling_mode = CastlingMode::detect(setup);
    let position = Chess::from_setup(setup.clone(), castling_mode).map_err(|_| ())?;

    Ok(Some(position))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionLevel {
    Low = 0,
    Medium = 1,
    High = 2,
}

const LEVELS: [CompressionLevel; 3] = [
    CompressionLevel::Low,
    CompressionLevel::Medium,
    CompressionLevel::High,
];

/// An event in a game stream, including variation markers.
#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    /// A chess move on the current line.
    Move(Move),
    /// Start of a sub-variation (branch from current position before the preceding move).
    StartVariation,
    /// End of the current sub-variation (return to parent line).
    EndVariation,
}

/// Encoder for chess games with different compression levels.
pub enum Encoder<'a> {
    Naive(NaiveEncoder),
    CompactIndex(CompactIndexEncoder),
    Huffman(HuffEncoder<'a>),
}

/// Encoded representation of a chess game.
#[derive(Clone, Debug)]
pub struct EncodedGame<'a> {
    content: EncodedGameContent<'a>,
    compression_level: CompressionLevel,
}

#[derive(Clone, Debug)]
pub(crate) enum EncodedGameContent<'a> {
    Bytes(Cow<'a, [u8]>),
    Bits(BitsEncodedGame),
}

/// Error type for constructing an `EncodedGame` from bytes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodedGameConstructionError {
    EmptyData = 0,
    InvalidCompressionLevel = 1,
    InvalidData = 2,
}

impl std::error::Error for EncodedGameConstructionError {}

impl fmt::Display for EncodedGameConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EncodedGameConstructionError::EmptyData =>
                    "Cannot construct EncodedGame from empty data",
                EncodedGameConstructionError::InvalidCompressionLevel =>
                    "Invalid compression level in EncodedGame data",
                EncodedGameConstructionError::InvalidData =>
                    "Invalid data for constructing EncodedGame",
            }
        )
    }
}

impl From<chess_huffman::EncodedGameConstructionError> for EncodedGameConstructionError {
    fn from(e: chess_huffman::EncodedGameConstructionError) -> Self {
        match e {
            chess_huffman::EncodedGameConstructionError::EmptyBytes => {
                EncodedGameConstructionError::EmptyData
            }
            chess_huffman::EncodedGameConstructionError::InvalidBytes => {
                EncodedGameConstructionError::InvalidData
            }
        }
    }
}

impl<'a> EncodedGame<'a> {
    #[must_use]
    pub fn compression_level(&self) -> CompressionLevel {
        self.compression_level
    }

    /// Converts an encoded game into bytes. Use `from_bytes` to reconstruct.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes = match self.content {
            EncodedGameContent::Bytes(bytes) => bytes.into_owned(),
            EncodedGameContent::Bits(bits) => bits.to_bytes(),
        };
        if self.compression_level == CompressionLevel::Low {
            bytes.push(0);
        } else {
            let len_minus_one = bytes.len() - 1;
            let last_byte = bytes[len_minus_one];
            bytes[len_minus_one] = last_byte | (self.compression_level as u8) << 6;
        }

        bytes
    }

    /// Constructs an encoded game from a byte slice produced by `into_bytes`.
    #[must_use]
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, EncodedGameConstructionError> {
        if bytes.is_empty() {
            return Err(EncodedGameConstructionError::EmptyData);
        }

        let len_minus_one = bytes.len() - 1;

        let level_bits = bytes[len_minus_one] >> 6;
        if level_bits > 2 {
            return Err(EncodedGameConstructionError::InvalidCompressionLevel);
        }

        let level = LEVELS[(bytes[len_minus_one] >> 6) as usize];

        let content = if level == CompressionLevel::Low {
            EncodedGameContent::Bytes(Cow::Borrowed(&bytes[..len_minus_one]))
        } else {
            EncodedGameContent::Bits(BitsEncodedGame::from_bytes(bytes)?)
        };

        Ok(EncodedGame {
            content,
            compression_level: level,
        })
    }

    /// Constructs an encoded game from an owned byte vector produced by `into_bytes`.
    #[must_use]
    pub fn from_owned_bytes(mut bytes: Vec<u8>) -> Result<Self, EncodedGameConstructionError> {
        let len_minus_one = bytes.len() - 1;

        let level = LEVELS[(bytes[len_minus_one] >> 6) as usize];

        let content = if level == CompressionLevel::Low {
            bytes.pop();
            EncodedGameContent::Bytes(Cow::Owned(bytes))
        } else {
            EncodedGameContent::Bits(BitsEncodedGame::from_bytes(&bytes)?)
        };

        Ok(EncodedGame {
            content,
            compression_level: level,
        })
    }

    /// Recompresses the encoded game into a different compression level.
    #[must_use]
    pub fn recompress(self, level: CompressionLevel) -> DecodeResult<Self> {
        let mut encoder = Encoder::new(level);
        let mut decoder = Decoder::new(&self);
        while let Some(m) = decoder.next_move() {
            encoder.encode_move(m?).expect("Encoding in recompress() failed, which should not happen because decoding succeeded");
        }
        Ok(encoder.finish())
    }
}

impl Encoder<'_> {
    /// Creates a new encoder for the specified compression level.
    #[must_use]
    pub fn new(compression_level: CompressionLevel) -> Self {
        Self::new_with_initial_fen(compression_level, None)
            .expect("None as initial FEN should always be valid")
    }

    /// Creates a new encoder for the specified compression level and optional initial FEN.
    pub fn new_with_initial_fen(
        compression_level: CompressionLevel,
        initial_fen: Option<&str>,
    ) -> Result<Self, EncodeError> {
        match compression_level {
            CompressionLevel::Low => {
                let initial_position = parse_initial_position(initial_fen)
                    .map_err(|_| EncodeError::from_inner("invalid initial FEN"))?;
                Ok(Encoder::Naive(NaiveEncoder::new(initial_position)))
            }
            CompressionLevel::Medium => Ok(Encoder::CompactIndex(CompactIndexEncoder::new())),
            CompressionLevel::High => Ok(Encoder::Huffman(HuffEncoder::new())),
        }
    }

    /// Encodes a start-variation sentinel. Only supported for Low compression.
    /// Panics if called on a non-Naive encoder.
    pub fn encode_start_variation(&mut self) {
        match self {
            Encoder::Naive(enc) => enc.encode_start_variation(),
            _ => panic!("encode_start_variation is only supported for Low compression"),
        }
    }

    /// Encodes an end-variation sentinel. Only supported for Low compression.
    /// Panics if called on a non-Naive encoder.
    pub fn encode_end_variation(&mut self) {
        match self {
            Encoder::Naive(enc) => enc.encode_end_variation(),
            _ => panic!("encode_end_variation is only supported for Low compression"),
        }
    }
}

impl Encode for Encoder<'_> {
    fn encode_move(&mut self, m: Move) -> Result<(), EncodeError> {
        match self {
            Encoder::Naive(enc) => enc.encode_move(m),
            Encoder::CompactIndex(enc) => enc.encode_move(m),
            Encoder::Huffman(enc) => enc.encode_move(m),
        }
    }

    fn finish(self) -> EncodedGame<'static> {
        match self {
            Encoder::Naive(enc) => enc.finish(),
            Encoder::CompactIndex(enc) => enc.finish(),
            Encoder::Huffman(enc) => enc.finish(),
        }
    }
}

pub trait Encode {
    /// Encodes a move into the game.
    fn encode_move(&mut self, m: Move) -> Result<(), EncodeError>;
    /// Finalizes the encoding and returns the encoded game.
    fn finish(self) -> EncodedGame<'static>;
}

pub enum Decoder<'a> {
    Naive(NaiveDecoder<'a>),
    CompactIndex(CompactIndexDecoder<'a>),
    Huffman(HuffDecoder<'a>),
}

impl<'a> Decoder<'a> {
    /// Creates a new decoder for an encoded game.
    #[must_use]
    pub fn new(encoded: &'a EncodedGame) -> Self {
        Self::new_with_initial_fen(encoded, None).expect("None as initial FEN should always be valid")
    }

    /// Creates a new decoder for an encoded game and optional initial FEN.
    pub fn new_with_initial_fen(
        encoded: &'a EncodedGame,
        initial_fen: Option<&str>,
    ) -> DecodeResult<Self> {
        match encoded.compression_level {
            CompressionLevel::Low => {
                let initial_position = parse_initial_position(initial_fen).map_err(|_| DecodeError {})?;
                Ok(Decoder::Naive(NaiveDecoder::new(
                    &encoded.content,
                    initial_position,
                )))
            }
            CompressionLevel::Medium => {
                Ok(Decoder::CompactIndex(CompactIndexDecoder::new(&encoded.content)))
            }
            CompressionLevel::High => Ok(Decoder::Huffman(HuffDecoder::new(&encoded.content))),
        }
    }

    fn initial_position(&self) -> Chess {
        match self {
            Decoder::Naive(decoder) => decoder.initial_position().clone(),
            Decoder::CompactIndex(_) | Decoder::Huffman(_) => Chess::new(),
        }
    }

    /// Decodes all moves and represents the game as a UCI string.
    pub fn into_uci_string(self) -> DecodeResult<String>
    where
        Self: Sized,
    {
        let mut s = String::new();
        let mut first = true;
        for m in self.into_iter_moves() {
            if !first {
                s.push(' ');
            }
            first = false;
            s.push_str(&UciMove::from_standard(m?).to_string());
        }
        Ok(s)
    }

    /// Decodes all moves and represents the game as a PGN string (mainline only).
    pub fn into_pgn_string(self) -> DecodeResult<String>
    where
        Self: Sized,
    {
        let mut s = String::new();
        let mut pos = self.initial_position();
        let mut first = true;
        let mut i = 2;
        for r in self.into_iter_moves_and_positions() {
            let (m, next_pos) = r?;
            if !first {
                s.push(' ');
            }
            first = false;

            if i % 2 == 0 {
                let move_number = i / 2;
                s.push_str(&format!("{}. ", move_number));
            }
            i += 1;

            let san = San::from_move(&pos, m);
            let suffix = Suffix::from_position(&next_pos);
            let san_plus = SanPlus { san, suffix };
            san_plus.append_to_string(&mut s);

            pos = next_pos;
        }

        Ok(s)
    }

    /// Decodes all moves including sub-variations and represents the game as a PGN string.
    /// For Low-compression games that may contain variations, this produces output
    /// with nested `(...)` notation. For other compression levels, this is identical
    /// to `into_pgn_string()` since they don't support variations.
    pub fn into_pgn_string_with_variations(self) -> DecodeResult<String>
    where
        Self: Sized,
    {
        match self {
            Decoder::Naive(decoder) => Self::naive_pgn_with_variations(decoder),
            // Other decoders don't support variations, fall back to mainline PGN.
            other => other.into_pgn_string(),
        }
    }

    /// Internal: build PGN string from a NaiveDecoder using next_event(),
    /// handling sub-variations with a position stack.
    fn naive_pgn_with_variations(mut decoder: NaiveDecoder<'_>) -> DecodeResult<String> {
        let mut s = String::new();
        let initial_pos = decoder.initial_position().clone();

        // Stack of saved state when entering a variation.
        // When we enter a variation, we save the parent state so we can restore it on EndVariation.
        struct Frame {
            pos_before_branch_move: Chess,
            ply: u32,
        }

        let mut stack: Vec<Frame> = vec![];
        let mut pos = initial_pos;
        let mut ply: u32 = 0; // 0-based: 0 = white's first move, 1 = black's first move, etc.
        let mut first_in_line = true;
        let mut needs_space = false;
        // Track the position BEFORE the last move played in the current line,
        // so that when StartVariation arrives we can branch from there.
        let mut pos_before_last_move: Option<Chess> = None;

        loop {
            let event = match decoder.next_event() {
                None => break,
                Some(Ok(ev)) => ev,
                Some(Err(e)) => return Err(e),
            };

            match event {
                GameEvent::Move(m) => {
                    if needs_space {
                        s.push(' ');
                    }

                    // White move: always print move number.
                    // Black move: print move number with "..." only if first in line.
                    if ply % 2 == 0 {
                        let move_number = ply / 2 + 1;
                        s.push_str(&format!("{}. ", move_number));
                    } else if first_in_line {
                        let move_number = ply / 2 + 1;
                        s.push_str(&format!("{}... ", move_number));
                    }

                    let san = San::from_move(&pos, m);
                    pos_before_last_move = Some(pos.clone());
                    pos.play_unchecked(m);
                    let suffix = Suffix::from_position(&pos);
                    let san_plus = SanPlus { san, suffix };
                    san_plus.append_to_string(&mut s);

                    ply += 1;
                    first_in_line = false;
                    needs_space = true;
                }
                GameEvent::StartVariation => {
                    // Save current state. The variation branches from the position
                    // before the last move on this line.
                    let branch_pos = pos_before_last_move
                        .clone()
                        .expect("StartVariation before any move");
                    let branch_ply = ply - 1; // the ply the variation starts at

                    stack.push(Frame {
                        pos_before_branch_move: pos.clone(),
                        ply,
                    });

                    // Start the variation
                    if needs_space {
                        s.push(' ');
                    }
                    s.push('(');
                    needs_space = false;
                    pos = branch_pos;
                    decoder.set_position(pos.clone());
                    ply = branch_ply;
                    first_in_line = true;
                }
                GameEvent::EndVariation => {
                    s.push(')');
                    needs_space = true;

                    let frame = stack.pop().expect("EndVariation without matching StartVariation");
                    pos = frame.pos_before_branch_move;
                    decoder.set_position(pos.clone());
                    ply = frame.ply;
                    // After a variation closes, the next move always needs a move number
                    // indicator in PGN, regardless of what the parent state was.
                    first_in_line = true;
                    pos_before_last_move = None; // reset; next move will set it
                }
            }
        }

        Ok(s)
    }

    /// Decodes all moves and positions into vectors.
    pub fn decode_all_moves_and_positions(self) -> DecodeResult<(Vec<Move>, Vec<Chess>)> {
        let mut moves = vec![];
        let mut positions = vec![];

        for d in self.into_iter_moves_and_positions() {
            let (m, pos) = d?;
            moves.push(m);
            positions.push(pos);
        }

        Ok((moves, positions))
    }
}

impl Decode for Decoder<'_> {
    fn next_move(&mut self) -> Option<DecodeResult<Move>> {
        match self {
            Decoder::Naive(decoder) => decoder.next_move(),
            Decoder::CompactIndex(decoder) => decoder.next_move(),
            Decoder::Huffman(decoder) => decoder.next_move(),
        }
    }

    fn next_position(&mut self) -> Option<DecodeResult<&Chess>> {
        match self {
            Decoder::Naive(decoder) => decoder.next_position(),
            Decoder::CompactIndex(decoder) => decoder.next_position(),
            Decoder::Huffman(decoder) => decoder.next_position(),
        }
    }

    fn next_move_and_position(&mut self) -> Option<DecodeResult<(Move, &Chess)>> {
        match self {
            Decoder::Naive(decoder) => decoder.next_move_and_position(),
            Decoder::CompactIndex(decoder) => decoder.next_move_and_position(),
            Decoder::Huffman(decoder) => decoder.next_move_and_position(),
        }
    }
}

pub trait Decode {
    /// Decodes the next move and returns it.
    fn next_move(&mut self) -> Option<DecodeResult<Move>>;
    /// Decodes the next move and returns the position after it.
    fn next_position(&mut self) -> Option<DecodeResult<&Chess>>;
    /// Decodes the next move and returns it along with the position after it.
    fn next_move_and_position(&mut self) -> Option<DecodeResult<(Move, &Chess)>>;

    /// Converts the decoder into an iterator over moves.
    fn into_iter_moves(self) -> impl Iterator<Item = DecodeResult<Move>>
    where
        Self: Sized,
    {
        struct MoveIter<T> {
            decoder: T,
            error: bool,
        }
        impl<T: Decode> Iterator for MoveIter<T> {
            type Item = DecodeResult<Move>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.error {
                    return Some(Err(DecodeError {}));
                }

                let item = self.decoder.next_move();
                if let Some(Err(_)) = item {
                    self.error = true;
                }
                item
            }
        }

        MoveIter {
            decoder: self,
            error: false,
        }
    }

    /// Converts the decoder into an iterator over positions.
    fn into_iter_positions(self) -> impl Iterator<Item = DecodeResult<Chess>>
    where
        Self: Sized,
    {
        struct PosIter<T> {
            decoder: T,
            error: bool,
        }
        impl<T: Decode> Iterator for PosIter<T> {
            type Item = DecodeResult<Chess>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.error {
                    return Some(Err(DecodeError {}));
                }

                let item = self.decoder.next_position().map(|res| res.cloned());
                if let Some(Err(_)) = item {
                    self.error = true;
                }
                item
            }
        }

        PosIter {
            decoder: self,
            error: false,
        }
    }

    /// Converts the decoder into an iterator over moves and positions.
    fn into_iter_moves_and_positions(self) -> impl Iterator<Item = DecodeResult<(Move, Chess)>>
    where
        Self: Sized,
    {
        struct MovePosIter<T> {
            decoder: T,
            error: bool,
        }
        impl<T: Decode> Iterator for MovePosIter<T> {
            type Item = DecodeResult<(Move, Chess)>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.error {
                    return Some(Err(DecodeError {}));
                }

                let item = self
                    .decoder
                    .next_move_and_position()
                    .map(|r| r.map(|(m, p)| (m, p.clone())));
                if let Some(Err(_)) = item {
                    self.error = true;
                }
                item
            }
        }

        MovePosIter {
            decoder: self,
            error: false,
        }
    }
}

/// Error type for decoding failures.
#[derive(Clone, Debug)]
pub struct DecodeError {}

impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot decode invalid game data")
    }
}

/// Error type for decoding failures.
#[derive(Debug)]
pub struct EncodeError {
    inner: Box<dyn std::fmt::Debug + Send + Sync>,
}

impl std::error::Error for EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to encode move: {:?}", self.inner)
    }
}

impl EncodeError {
    fn from_inner<E: std::fmt::Debug + Send + Sync + 'static>(err: E) -> Self {
        EncodeError {
            inner: Box::new(err),
        }
    }
}

/// Result type for decoding operations.
pub type DecodeResult<T> = Result<T, DecodeError>;

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use shakmaty::{
        CastlingMode, Chess, FromSetup, Move, Position,
        fen::Fen as ShakmatyFen,
    };

    use crate::Decode;

    use super::{CompressionLevel, Decoder, Encode, EncodedGame, Encoder};

    fn random_games_consistency(move_ids: Vec<u16>, level: CompressionLevel) -> bool {
        let mut pos = Chess::default();
        let mut moves: Vec<Move> = vec![];
        let mut positions: Vec<Chess> = vec![];

        for m in move_ids {
            let legal_moves = pos.legal_moves();
            if legal_moves.is_empty() {
                break;
            }

            let i = m as usize % legal_moves.len();
            let choice = legal_moves[i];
            pos.play_unchecked(choice);
            moves.push(choice);
            positions.push(pos.clone());
        }

        let mut encoder = Encoder::new(level);
        for &m in &moves {
            encoder.encode_move(m).unwrap();
        }

        let encoded = encoder.finish();
        if encoded.compression_level != level {
            panic!("encoded.compression_level != level");
        }

        let bytes = encoded.clone().into_bytes();

        let restored = EncodedGame::from_bytes(&bytes).unwrap();
        if restored.compression_level != encoded.compression_level {
            panic!("restored.compression_level != encoded.compression_level");
        }

        let decoder = Decoder::new(&restored);
        let restored_moves: Vec<Move> = decoder.into_iter_moves().map(|m| m.unwrap()).collect();

        let decoder2 = Decoder::new(&restored);
        let restored_positions: Vec<Chess> =
            decoder2.into_iter_positions().map(|p| p.unwrap()).collect();

        if moves != restored_moves {
            panic!("restored_moves != moves");
        }

        if positions != restored_positions {
            panic!("restored_positions != positions");
        }

        true
    }

    #[quickcheck]
    fn random_games_consistency_low(move_ids: Vec<u16>) -> bool {
        random_games_consistency(move_ids, CompressionLevel::Low)
    }

    #[quickcheck]
    fn random_games_consistency_medium(move_ids: Vec<u16>) -> bool {
        random_games_consistency(move_ids, CompressionLevel::Medium)
    }

    #[quickcheck]
    fn random_games_consistency_high(move_ids: Vec<u16>) -> bool {
        random_games_consistency(move_ids, CompressionLevel::High)
    }

    #[quickcheck]
    fn no_decode_panics(data: Vec<u8>) -> bool {
        match EncodedGame::from_bytes(&data) {
            Ok(encoded) => {
                let mut decoder = Decoder::new(&encoded);
                while let Some(m) = decoder.next_move() {
                    assert!(m.is_ok() || m.is_err());
                    if m.is_err() {
                        break;
                    }
                }
            }
            Err(_) => {}
        }
        true
    }

    #[test]
    fn low_custom_fen_roundtrip() {
        let fen = "7k/8/8/8/8/8/8/K7 w - - 0 1";
        let parsed = ShakmatyFen::from_ascii(fen.as_bytes()).unwrap();
        let setup = parsed.as_setup();
        let castling_mode = CastlingMode::detect(setup);
        let mut pos = Chess::from_setup(setup.clone(), castling_mode).unwrap();

        let mut moves = vec![];
        for _ in 0..3 {
            let legal_moves = pos.legal_moves();
            let m = legal_moves[0];
            pos.play_unchecked(m);
            moves.push(m);
        }

        let mut encoder = Encoder::new_with_initial_fen(CompressionLevel::Low, Some(fen)).unwrap();
        for m in &moves {
            encoder.encode_move(*m).unwrap();
        }
        let encoded = encoder.finish();

        let decoder = Decoder::new_with_initial_fen(&encoded, Some(fen)).unwrap();
        let decoded_moves: Vec<Move> = decoder.into_iter_moves().map(|m| m.unwrap()).collect();
        assert_eq!(decoded_moves, moves);

        let mut wrong_decoder = Decoder::new(&encoded);
        assert!(wrong_decoder.next_move().unwrap().is_err());
    }

    #[test]
    fn pgn_with_simple_variation() {
        // 1. e4 (1. d4) 1... e5
        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4").unwrap().to_move(&pos).unwrap();
        let d4 = shakmaty::san::San::from_ascii(b"d4").unwrap().to_move(&pos).unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);
        let e5 = shakmaty::san::San::from_ascii(b"e5").unwrap().to_move(&pos_after_e4).unwrap();

        let mut encoder = Encoder::new(CompressionLevel::Low);
        encoder.encode_move(e4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d4).unwrap();
        encoder.encode_end_variation();
        encoder.encode_move(e5).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();

        // Mainline PGN should not include variations
        let decoder_mainline = Decoder::new(&restored);
        let mainline_pgn = decoder_mainline.into_pgn_string().unwrap();
        assert_eq!(mainline_pgn, "1. e4 e5");

        // Full PGN should include variations
        let decoder_full = Decoder::new(&restored);
        let full_pgn = decoder_full.into_pgn_string_with_variations().unwrap();
        assert_eq!(full_pgn, "1. e4 (1. d4) 1... e5");
    }

    #[test]
    fn pgn_with_nested_variation() {
        // 1. e4 (1. d4 (1. c4)) 1... e5 2. Nf3
        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4").unwrap().to_move(&pos).unwrap();
        let d4 = shakmaty::san::San::from_ascii(b"d4").unwrap().to_move(&pos).unwrap();
        let c4 = shakmaty::san::San::from_ascii(b"c4").unwrap().to_move(&pos).unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);
        let e5 = shakmaty::san::San::from_ascii(b"e5").unwrap().to_move(&pos_after_e4).unwrap();

        let mut pos_after_e4_e5 = pos_after_e4.clone();
        pos_after_e4_e5.play_unchecked(e5);
        let nf3 = shakmaty::san::San::from_ascii(b"Nf3").unwrap().to_move(&pos_after_e4_e5).unwrap();

        let mut encoder = Encoder::new(CompressionLevel::Low);
        encoder.encode_move(e4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(c4).unwrap();
        encoder.encode_end_variation();
        encoder.encode_end_variation();
        encoder.encode_move(e5).unwrap();
        encoder.encode_move(nf3).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();

        // Mainline: 1. e4 e5 2. Nf3
        let decoder_mainline = Decoder::new(&restored);
        let mainline_pgn = decoder_mainline.into_pgn_string().unwrap();
        assert_eq!(mainline_pgn, "1. e4 e5 2. Nf3");

        // Full PGN
        let decoder_full = Decoder::new(&restored);
        let full_pgn = decoder_full.into_pgn_string_with_variations().unwrap();
        assert_eq!(full_pgn, "1. e4 (1. d4 (1. c4)) 1... e5 2. Nf3");
    }

    #[test]
    fn pgn_variation_on_black_move() {
        // 1. e4 e5 (1... d5) 2. Nf3
        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4").unwrap().to_move(&pos).unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);
        let e5 = shakmaty::san::San::from_ascii(b"e5").unwrap().to_move(&pos_after_e4).unwrap();
        let d5 = shakmaty::san::San::from_ascii(b"d5").unwrap().to_move(&pos_after_e4).unwrap();

        let mut pos_after_e4_e5 = pos_after_e4.clone();
        pos_after_e4_e5.play_unchecked(e5);
        let nf3 = shakmaty::san::San::from_ascii(b"Nf3").unwrap().to_move(&pos_after_e4_e5).unwrap();

        let mut encoder = Encoder::new(CompressionLevel::Low);
        encoder.encode_move(e4).unwrap();
        encoder.encode_move(e5).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d5).unwrap();
        encoder.encode_end_variation();
        encoder.encode_move(nf3).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();

        // Full PGN
        let decoder_full = Decoder::new(&restored);
        let full_pgn = decoder_full.into_pgn_string_with_variations().unwrap();
        assert_eq!(full_pgn, "1. e4 e5 (1... d5) 2. Nf3");
    }
}
