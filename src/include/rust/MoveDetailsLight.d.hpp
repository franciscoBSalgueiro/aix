#ifndef MoveDetailsLight_D_HPP
#define MoveDetailsLight_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct MoveDetailsLight {
      uint16_t ply;
      int8_t role;
      uint8_t from;
      uint8_t to;
      int8_t capture;
      bool is_castle;
      int8_t promotion;
      bool is_en_passant;
    };
    
    typedef struct MoveDetailsLight_option {union { MoveDetailsLight ok; }; bool is_ok; } MoveDetailsLight_option;
} // namespace capi
} // namespace


struct MoveDetailsLight {
  uint16_t ply;
  int8_t role;
  uint8_t from;
  uint8_t to;
  int8_t capture;
  bool is_castle;
  int8_t promotion;
  bool is_en_passant;

  inline diplomat::capi::MoveDetailsLight AsFFI() const;
  inline static MoveDetailsLight FromFFI(diplomat::capi::MoveDetailsLight c_struct);
};


#endif // MoveDetailsLight_D_HPP
