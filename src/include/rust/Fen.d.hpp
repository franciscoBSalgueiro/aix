#ifndef Fen_D_HPP
#define Fen_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"

class DecodeError;


namespace diplomat {
namespace capi {
    struct Fen {
      uint64_t white;
      uint64_t black;
      uint64_t king;
      uint64_t queen;
      uint64_t rook;
      uint64_t bishop;
      uint64_t knight;
      uint64_t pawn;
      bool white_to_move;
      uint64_t castling_rights;
      int8_t ep_square;
    };

    typedef struct Fen_option {union { Fen ok; }; bool is_ok; } Fen_option;
} // namespace capi
} // namespace


struct Fen {
  uint64_t white;
  uint64_t black;
  uint64_t king;
  uint64_t queen;
  uint64_t rook;
  uint64_t bishop;
  uint64_t knight;
  uint64_t pawn;
  bool white_to_move;
  uint64_t castling_rights;
  int8_t ep_square;

  inline static diplomat::result<Fen, std::monostate> parse(std::string_view fen);

  inline diplomat::result<bool, DecodeError> matches_fen(diplomat::span<const uint8_t> game);

  inline diplomat::result<uint16_t, DecodeError> matches_fen_ply(diplomat::span<const uint8_t> game);

  inline diplomat::result<bool, DecodeError> matches_fen_from_fen(diplomat::span<const uint8_t> game, std::string_view initial_fen);

  inline diplomat::result<uint16_t, DecodeError> matches_fen_ply_from_fen(diplomat::span<const uint8_t> game, std::string_view initial_fen);

  inline diplomat::capi::Fen AsFFI() const;
  inline static Fen FromFFI(diplomat::capi::Fen c_struct);
};


#endif // Fen_D_HPP
