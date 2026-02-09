#ifndef MoveDetailsFull_HPP
#define MoveDetailsFull_HPP

#include "MoveDetailsFull.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {
    
    
    } // extern "C"
} // namespace capi
} // namespace


inline diplomat::capi::MoveDetailsFull MoveDetailsFull::AsFFI() const {
  return diplomat::capi::MoveDetailsFull {
    /* .ply = */ ply,
    /* .role = */ role,
    /* .from = */ from,
    /* .to = */ to,
    /* .capture = */ capture,
    /* .is_castle = */ is_castle,
    /* .promotion = */ promotion,
    /* .is_check = */ is_check,
    /* .is_checkmate = */ is_checkmate,
    /* .is_stalemate = */ is_stalemate,
    /* .is_en_passant = */ is_en_passant,
  };
}

inline MoveDetailsFull MoveDetailsFull::FromFFI(diplomat::capi::MoveDetailsFull c_struct) {
  return MoveDetailsFull {
    /* .ply = */ c_struct.ply,
    /* .role = */ c_struct.role,
    /* .from = */ c_struct.from,
    /* .to = */ c_struct.to,
    /* .capture = */ c_struct.capture,
    /* .is_castle = */ c_struct.is_castle,
    /* .promotion = */ c_struct.promotion,
    /* .is_check = */ c_struct.is_check,
    /* .is_checkmate = */ c_struct.is_checkmate,
    /* .is_stalemate = */ c_struct.is_stalemate,
    /* .is_en_passant = */ c_struct.is_en_passant,
  };
}


#endif // MoveDetailsFull_HPP
