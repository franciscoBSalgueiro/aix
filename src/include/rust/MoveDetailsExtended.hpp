#ifndef MoveDetailsExtended_HPP
#define MoveDetailsExtended_HPP

#include "MoveDetailsExtended.d.hpp"

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


inline diplomat::capi::MoveDetailsExtended MoveDetailsExtended::AsFFI() const {
  return diplomat::capi::MoveDetailsExtended {
    /* .ply = */ ply,
    /* .role = */ role,
    /* .from = */ from,
    /* .to = */ to,
    /* .capture = */ capture,
    /* .is_castle = */ is_castle,
    /* .promotion = */ promotion,
    /* .is_en_passant = */ is_en_passant,
    /* .is_check = */ is_check,
    /* .is_checkmate = */ is_checkmate,
    /* .is_stalemate = */ is_stalemate,
    /* .legal_response_move_count = */ legal_response_move_count,
  };
}

inline MoveDetailsExtended MoveDetailsExtended::FromFFI(diplomat::capi::MoveDetailsExtended c_struct) {
  return MoveDetailsExtended {
    /* .ply = */ c_struct.ply,
    /* .role = */ c_struct.role,
    /* .from = */ c_struct.from,
    /* .to = */ c_struct.to,
    /* .capture = */ c_struct.capture,
    /* .is_castle = */ c_struct.is_castle,
    /* .promotion = */ c_struct.promotion,
    /* .is_en_passant = */ c_struct.is_en_passant,
    /* .is_check = */ c_struct.is_check,
    /* .is_checkmate = */ c_struct.is_checkmate,
    /* .is_stalemate = */ c_struct.is_stalemate,
    /* .legal_response_move_count = */ c_struct.legal_response_move_count,
  };
}


#endif // MoveDetailsExtended_HPP
