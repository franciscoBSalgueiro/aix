#ifndef MoveDetailsExtended_D_HPP
#define MoveDetailsExtended_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct MoveDetailsExtended {
      uint16_t ply;
      int8_t role;
      uint8_t from;
      uint8_t to;
      int8_t capture;
      bool is_castle;
      int8_t promotion;
      bool is_en_passant;
      bool is_check;
      bool is_checkmate;
      bool is_stalemate;
      uint8_t legal_response_move_count;
    };
    
    typedef struct MoveDetailsExtended_option {union { MoveDetailsExtended ok; }; bool is_ok; } MoveDetailsExtended_option;
} // namespace capi
} // namespace


struct MoveDetailsExtended {
  uint16_t ply;
  int8_t role;
  uint8_t from;
  uint8_t to;
  int8_t capture;
  bool is_castle;
  int8_t promotion;
  bool is_en_passant;
  bool is_check;
  bool is_checkmate;
  bool is_stalemate;
  uint8_t legal_response_move_count;

  inline diplomat::capi::MoveDetailsExtended AsFFI() const;
  inline static MoveDetailsExtended FromFFI(diplomat::capi::MoveDetailsExtended c_struct);
};


#endif // MoveDetailsExtended_D_HPP
