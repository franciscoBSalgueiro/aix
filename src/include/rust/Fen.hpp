#ifndef Fen_HPP
#define Fen_HPP

#include "Fen.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "DecodeError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Fen_parse_result {union {diplomat::capi::Fen ok; }; bool is_ok;} Fen_parse_result;
    Fen_parse_result Fen_parse(diplomat::capi::DiplomatStringView fen);

    typedef struct Fen_matches_fen_result {union {bool ok; diplomat::capi::DecodeError err;}; bool is_ok;} Fen_matches_fen_result;
    Fen_matches_fen_result Fen_matches_fen(diplomat::capi::Fen self, diplomat::capi::DiplomatU8View game);


    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<Fen, std::monostate> Fen::parse(std::string_view fen) {
  auto result = diplomat::capi::Fen_parse({fen.data(), fen.size()});
  return result.is_ok ? diplomat::result<Fen, std::monostate>(diplomat::Ok<Fen>(Fen::FromFFI(result.ok))) : diplomat::result<Fen, std::monostate>(diplomat::Err<std::monostate>());
}

inline diplomat::result<bool, DecodeError> Fen::matches_fen(diplomat::span<const uint8_t> game) {
  auto result = diplomat::capi::Fen_matches_fen(this->AsFFI(),
    {game.data(), game.size()});
  return result.is_ok ? diplomat::result<bool, DecodeError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}


inline diplomat::capi::Fen Fen::AsFFI() const {
  return diplomat::capi::Fen {
    /* .white = */ white,
    /* .black = */ black,
    /* .king = */ king,
    /* .queen = */ queen,
    /* .rook = */ rook,
    /* .bishop = */ bishop,
    /* .knight = */ knight,
    /* .pawn = */ pawn,
    /* .white_to_move = */ white_to_move,
    /* .castling_rights = */ castling_rights,
    /* .ep_square = */ ep_square,
  };
}

inline Fen Fen::FromFFI(diplomat::capi::Fen c_struct) {
  return Fen {
    /* .white = */ c_struct.white,
    /* .black = */ c_struct.black,
    /* .king = */ c_struct.king,
    /* .queen = */ c_struct.queen,
    /* .rook = */ c_struct.rook,
    /* .bishop = */ c_struct.bishop,
    /* .knight = */ c_struct.knight,
    /* .pawn = */ c_struct.pawn,
    /* .white_to_move = */ c_struct.white_to_move,
    /* .castling_rights = */ c_struct.castling_rights,
    /* .ep_square = */ c_struct.ep_square,
  };
}


#endif // Fen_HPP
