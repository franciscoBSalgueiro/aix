#ifndef MoveDetailsFull_D_HPP
#define MoveDetailsFull_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct MoveDetailsFull {
      uint16_t ply;
      int8_t role;
      uint8_t from;
      uint8_t to;
      int8_t capture;
      bool is_castle;
      int8_t promotion;
      bool is_check;
      bool is_checkmate;
      bool is_stalemate;
      bool is_en_passant;
    };
    
    typedef struct MoveDetailsFull_option {union { MoveDetailsFull ok; }; bool is_ok; } MoveDetailsFull_option;
} // namespace capi
} // namespace


struct MoveDetailsFull {
  uint16_t ply;
  int8_t role;
  uint8_t from;
  uint8_t to;
  int8_t capture;
  bool is_castle;
  int8_t promotion;
  bool is_check;
  bool is_checkmate;
  bool is_stalemate;
  bool is_en_passant;

  inline diplomat::capi::MoveDetailsFull AsFFI() const;
  inline static MoveDetailsFull FromFFI(diplomat::capi::MoveDetailsFull c_struct);
};


#endif // MoveDetailsFull_D_HPP
