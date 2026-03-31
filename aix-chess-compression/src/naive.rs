use std::borrow::Cow;

use crate::{
    CompressionLevel, Decode, DecodeError, DecodeResult, Encode, EncodeError, EncodedGame,
    EncodedGameContent, GameEvent,
};
use shakmaty::{Chess, Move, Position, Square, uci::UciMove};

/// Sentinel byte pair: start of a sub-variation.
const SENTINEL_START_VARIATION: [u8; 2] = [0xFF, 0xFF];
/// Sentinel byte pair: end of a sub-variation.
const SENTINEL_END_VARIATION: [u8; 2] = [0xFF, 0xFE];

/// Returns true if b1 is a sentinel prefix byte.
#[inline]
fn is_sentinel(b1: u8) -> bool {
    // 0xFF as the first byte of a move pair is impossible in valid move data:
    // from=63 (h8) with both capture and promotion flags set cannot be a legal move.
    b1 == 0xFF
}

pub struct NaiveEncoder {
    result: Vec<u8>,
    _initial_position: Chess,
}

impl NaiveEncoder {
    pub fn new(initial_position: Option<Chess>) -> Self {
        Self {
            result: Vec::with_capacity(40),
            _initial_position: initial_position.unwrap_or_else(Chess::new),
        }
    }

    /// Emits a START_VARIATION sentinel into the byte stream.
    pub fn encode_start_variation(&mut self) {
        self.result.extend_from_slice(&SENTINEL_START_VARIATION);
    }

    /// Emits an END_VARIATION sentinel into the byte stream.
    pub fn encode_end_variation(&mut self) {
        self.result.extend_from_slice(&SENTINEL_END_VARIATION);
    }
}

impl Default for NaiveEncoder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Encode for NaiveEncoder {
    fn encode_move(&mut self, m: Move) -> Result<(), EncodeError> {
        let from: u8 = m
            .from()
            .ok_or(EncodeError {
                inner: Box::new("missing from square in NaiveEncoder::encode_move"),
            })?
            .into();
        let to: u8 = m.to().into();

        let mut b1 = from;
        let mut b2 = to;

        if m.is_capture() {
            b1 |= 0b1000_0000;
        }

        if let Some(promotion) = m.promotion() {
            b1 |= 0b0100_0000;
            b2 |= match promotion {
                shakmaty::Role::Queen => 0b0000_0000,
                shakmaty::Role::Rook => 0b0100_0000,
                shakmaty::Role::Bishop => 0b1000_0000,
                shakmaty::Role::Knight => 0b1100_0000,
                _ => panic!("Invalid promotion piece"),
            };
        }

        self.result.push(b1);
        self.result.push(b2);

        Ok(())
    }

    fn finish(self) -> EncodedGame<'static> {
        EncodedGame {
            content: EncodedGameContent::Bytes(Cow::Owned(self.result)),
            compression_level: CompressionLevel::Low,
        }
    }
}

pub struct NaiveDecoder<'a> {
    encoded: &'a [u8],
    index: usize,
    initial_position: Chess,
    chess: Chess,
}

impl<'a> NaiveDecoder<'a> {
    pub(crate) fn new(encoded: &'a EncodedGameContent<'a>, initial_position: Option<Chess>) -> Self {
        let initial_position = initial_position.unwrap_or_else(Chess::new);
        if let EncodedGameContent::Bytes(enc) = encoded {
            Self {
                encoded: enc,
                index: 0,
                chess: initial_position.clone(),
                initial_position,
            }
        } else {
            panic!("NaiveDecoder only accepts EncodedGameRef::Bytes");
        }
    }

    pub(crate) fn initial_position(&self) -> &Chess {
        &self.initial_position
    }

    /// Peeks at the current byte pair and returns what kind of token it is,
    /// without advancing the index.
    fn peek_sentinel(&self) -> Option<SentinelKind> {
        if self.index + 1 >= self.encoded.len() {
            return None;
        }
        let b1 = self.encoded[self.index];
        let b2 = self.encoded[self.index + 1];
        if b1 == SENTINEL_START_VARIATION[0] && b2 == SENTINEL_START_VARIATION[1] {
            Some(SentinelKind::StartVariation)
        } else if b1 == SENTINEL_END_VARIATION[0] && b2 == SENTINEL_END_VARIATION[1] {
            Some(SentinelKind::EndVariation)
        } else if is_sentinel(b1) {
            Some(SentinelKind::Unknown)
        } else {
            None
        }
    }

    /// Skips over an entire sub-variation in the byte stream, handling nesting.
    /// The index should be positioned right after the START_VARIATION sentinel
    /// when this is called.
    fn skip_variation(&mut self) -> DecodeResult<()> {
        let mut depth: u32 = 1;
        while depth > 0 {
            if self.index + 1 >= self.encoded.len() {
                return Err(DecodeError {});
            }
            let b1 = self.encoded[self.index];
            let b2 = self.encoded[self.index + 1];
            self.index += 2;

            if b1 == SENTINEL_START_VARIATION[0] && b2 == SENTINEL_START_VARIATION[1] {
                depth += 1;
            } else if b1 == SENTINEL_END_VARIATION[0] && b2 == SENTINEL_END_VARIATION[1] {
                depth -= 1;
            }
            // Otherwise it's a move inside the variation — skip it.
        }
        Ok(())
    }

    /// Decodes the raw next move at the current position (no sentinel handling).
    fn decode_raw_move(&mut self) -> Option<DecodeResult<Move>> {
        if self.index + 1 >= self.encoded.len() {
            return if self.index == self.encoded.len() {
                None
            } else {
                Some(Err(DecodeError {}))
            };
        }

        let b1 = self.encoded[self.index];
        let b2 = self.encoded[self.index + 1];

        let from = unsafe { Square::new_unchecked(u32::from(b1 & 0b0011_1111)) };
        let to = unsafe { Square::new_unchecked(u32::from(b2 & 0b0011_1111)) };

        let promotion = if b1 & 0b0100_0000 != 0 {
            match b2 & 0b1100_0000 {
                0b0000_0000 => Some(shakmaty::Role::Queen),
                0b0100_0000 => Some(shakmaty::Role::Rook),
                0b1000_0000 => Some(shakmaty::Role::Bishop),
                0b1100_0000 => Some(shakmaty::Role::Knight),
                _ => unreachable!(),
            }
        } else {
            None
        };

        let uci = UciMove::Normal {
            from,
            to,
            promotion,
        };
        let r = uci.to_move(&self.chess).map_err(|_| DecodeError {});
        Some(r.map(|m| {
            self.chess.play_unchecked(m); // uci.to_move already checks legality
            self.index += 2;
            m
        }))
    }

    /// Returns the next event in the stream, including variation markers.
    /// This method does NOT skip variations — it returns them as events.
    /// The caller is responsible for managing position state across variations.
    pub fn next_event(&mut self) -> Option<DecodeResult<GameEvent>> {
        if self.index >= self.encoded.len() {
            return None;
        }

        if self.index + 1 >= self.encoded.len() {
            return Some(Err(DecodeError {}));
        }

        match self.peek_sentinel() {
            Some(SentinelKind::StartVariation) => {
                self.index += 2;
                Some(Ok(GameEvent::StartVariation))
            }
            Some(SentinelKind::EndVariation) => {
                self.index += 2;
                Some(Ok(GameEvent::EndVariation))
            }
            Some(SentinelKind::Unknown) => Some(Err(DecodeError {})),
            None => self
                .decode_raw_move()
                .map(|r| r.map(GameEvent::Move)),
        }
    }

    /// Sets the decoder's chess position (used by the variation-aware PGN builder
    /// to restore position when entering/leaving variations).
    pub fn set_position(&mut self, pos: Chess) {
        self.chess = pos;
    }

    /// Returns a reference to the current chess position.
    pub fn position(&self) -> &Chess {
        &self.chess
    }
}

enum SentinelKind {
    StartVariation,
    EndVariation,
    Unknown,
}

impl Decode for NaiveDecoder<'_> {
    /// Returns the next mainline move, skipping over any sub-variations.
    fn next_move(&mut self) -> Option<DecodeResult<Move>> {
        loop {
            if self.index >= self.encoded.len() {
                return None;
            }

            if self.index + 1 >= self.encoded.len() {
                return Some(Err(DecodeError {}));
            }

            match self.peek_sentinel() {
                Some(SentinelKind::StartVariation) => {
                    self.index += 2; // skip the sentinel
                    if let Err(e) = self.skip_variation() {
                        return Some(Err(e));
                    }
                    // Continue loop to get the next mainline move
                }
                Some(SentinelKind::EndVariation) => {
                    // We should not encounter EndVariation on the mainline.
                    // This indicates malformed data.
                    return Some(Err(DecodeError {}));
                }
                Some(SentinelKind::Unknown) => {
                    return Some(Err(DecodeError {}));
                }
                None => {
                    return self.decode_raw_move();
                }
            }
        }
    }

    fn next_move_and_position(&mut self) -> Option<DecodeResult<(Move, &Chess)>> {
        let maybe_next = self.next_move();
        maybe_next.map(|next| next.map(|m| (m, &self.chess)))
    }

    fn next_position(&mut self) -> Option<DecodeResult<&Chess>> {
        let maybe_next = self.next_move();
        maybe_next.map(|next| next.map(|_| &self.chess))
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompressionLevel, Decode, Decoder, Encode, EncodedGame, Encoder, GameEvent};
    use shakmaty::{Chess, Move, Position};

    #[test]
    fn decode_test() {
        let bytes = b"\x0C\x1C4$\x05\x1A9*\x06\x151)\x04\x07>-\x0A\x12=\"\x9A5\xBC5\x15&5>\x03\x11-#\x91#>=#5\x00";
        let encoded_game = EncodedGame::from_bytes(bytes).unwrap();
        assert_eq!(encoded_game.compression_level, CompressionLevel::Low);

        let decoder = Decoder::new(&encoded_game);

        if let Decoder::Naive(..) = &decoder {
            // expected
        } else {
            panic!("Decoder is not Naive variant");
        }

        let uci = decoder.into_uci_string().unwrap();
        let expected_uci = "e2e4 e7e5 f1c4 b8c6 g1f3 b7b6 e1g1 g8f6 c2c3 f8c5 c4f7 e8f7 f3g5 f7g8 d1b3 f6d5 b3d5 g8f8 d5f7";
        assert_eq!(uci, expected_uci);
    }

    /// Helper: play random moves and return the moves and final position
    fn play_random_moves(move_ids: &[u16]) -> Vec<Move> {
        let mut pos = Chess::default();
        let mut moves = vec![];
        for &m in move_ids {
            let legal_moves = pos.legal_moves();
            if legal_moves.is_empty() {
                break;
            }
            let i = m as usize % legal_moves.len();
            let choice = legal_moves[i];
            pos.play_unchecked(choice);
            moves.push(choice);
        }
        moves
    }

    #[test]
    fn variation_skipped_by_next_move() {
        // Encode: 1. e4 (1. d4 d5) 1... e5
        // Mainline should be: e4 e5
        use super::NaiveEncoder;

        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4")
            .unwrap()
            .to_move(&pos)
            .unwrap();
        let d4 = shakmaty::san::San::from_ascii(b"d4")
            .unwrap()
            .to_move(&pos)
            .unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);

        let e5 = shakmaty::san::San::from_ascii(b"e5")
            .unwrap()
            .to_move(&pos_after_e4)
            .unwrap();

        let mut pos_after_d4 = pos.clone();
        pos_after_d4.play_unchecked(d4);

        let d5 = shakmaty::san::San::from_ascii(b"d5")
            .unwrap()
            .to_move(&pos_after_d4)
            .unwrap();

        let mut encoder = NaiveEncoder::new(None);
        encoder.encode_move(e4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d4).unwrap(); // will not be valid position-wise in decoder but bytes are what matter for skip
        encoder.encode_move(d5).unwrap();
        encoder.encode_end_variation();
        encoder.encode_move(e5).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();
        let mut decoder = Decoder::new(&restored);

        // Mainline: e4, e5
        let m1 = decoder.next_move().unwrap().unwrap();
        assert_eq!(m1, e4);
        let m2 = decoder.next_move().unwrap().unwrap();
        assert_eq!(m2, e5);
        assert!(decoder.next_move().is_none());
    }

    #[test]
    fn variation_events_from_next_event() {
        use super::NaiveEncoder;

        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4")
            .unwrap()
            .to_move(&pos)
            .unwrap();
        let d4 = shakmaty::san::San::from_ascii(b"d4")
            .unwrap()
            .to_move(&pos)
            .unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);
        let e5 = shakmaty::san::San::from_ascii(b"e5")
            .unwrap()
            .to_move(&pos_after_e4)
            .unwrap();

        let mut encoder = NaiveEncoder::new(None);
        encoder.encode_move(e4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d4).unwrap();
        encoder.encode_end_variation();
        encoder.encode_move(e5).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();

        // Use Decoder to get the NaiveDecoder
        let decoder = Decoder::new(&restored);
        if let Decoder::Naive(mut naive_dec) = decoder {
            // Move 1: e4 (mainline)
            let ev1 = naive_dec.next_event().unwrap().unwrap();
            assert!(matches!(ev1, GameEvent::Move(_)));
            // Position is now after e4

            // StartVariation: save state and reset position to before e4
            let ev2 = naive_dec.next_event().unwrap().unwrap();
            assert!(matches!(ev2, GameEvent::StartVariation));
            naive_dec.set_position(pos.clone()); // reset to starting position (before e4)

            // Move inside variation: d4 (from starting position)
            let ev3 = naive_dec.next_event().unwrap().unwrap();
            assert!(matches!(ev3, GameEvent::Move(_)));

            // EndVariation: restore to after e4
            let ev4 = naive_dec.next_event().unwrap().unwrap();
            assert!(matches!(ev4, GameEvent::EndVariation));
            naive_dec.set_position(pos_after_e4.clone()); // restore to after e4

            // Move 2: e5 (mainline, from position after e4)
            let ev5 = naive_dec.next_event().unwrap().unwrap();
            assert!(matches!(ev5, GameEvent::Move(_)));

            assert!(naive_dec.next_event().is_none());
        } else {
            panic!("Expected Naive decoder");
        }
    }

    #[test]
    fn nested_variations_skipped() {
        use super::NaiveEncoder;

        let pos = Chess::default();
        let e4 = shakmaty::san::San::from_ascii(b"e4")
            .unwrap()
            .to_move(&pos)
            .unwrap();
        let d4 = shakmaty::san::San::from_ascii(b"d4")
            .unwrap()
            .to_move(&pos)
            .unwrap();
        let c4 = shakmaty::san::San::from_ascii(b"c4")
            .unwrap()
            .to_move(&pos)
            .unwrap();

        let mut pos_after_e4 = pos.clone();
        pos_after_e4.play_unchecked(e4);
        let e5 = shakmaty::san::San::from_ascii(b"e5")
            .unwrap()
            .to_move(&pos_after_e4)
            .unwrap();

        // Encode: e4 (d4 (c4)) e5
        // Mainline: e4 e5
        let mut encoder = NaiveEncoder::new(None);
        encoder.encode_move(e4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(d4).unwrap();
        encoder.encode_start_variation();
        encoder.encode_move(c4).unwrap();
        encoder.encode_end_variation();
        encoder.encode_end_variation();
        encoder.encode_move(e5).unwrap();
        let encoded = encoder.finish();

        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();
        let mut decoder = Decoder::new(&restored);

        let m1 = decoder.next_move().unwrap().unwrap();
        assert_eq!(m1, e4);
        let m2 = decoder.next_move().unwrap().unwrap();
        assert_eq!(m2, e5);
        assert!(decoder.next_move().is_none());
    }

    #[test]
    fn backward_compat_no_variations() {
        // A game with no variations should decode identically
        let moves = play_random_moves(&[3, 7, 12, 1, 9, 4]);

        let mut encoder = Encoder::new(CompressionLevel::Low);
        for &m in &moves {
            encoder.encode_move(m).unwrap();
        }
        let encoded = encoder.finish();
        let bytes = encoded.into_bytes();
        let restored = EncodedGame::from_bytes(&bytes).unwrap();
        let decoder = Decoder::new(&restored);
        let decoded_moves: Vec<Move> = decoder.into_iter_moves().map(|m| m.unwrap()).collect();
        assert_eq!(decoded_moves, moves);
    }
}
