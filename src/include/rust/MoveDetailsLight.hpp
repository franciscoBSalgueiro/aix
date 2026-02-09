#ifndef MoveDetailsLight_HPP
#define MoveDetailsLight_HPP

#include "MoveDetailsLight.d.hpp"

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


inline diplomat::capi::MoveDetailsLight MoveDetailsLight::AsFFI() const {
  return diplomat::capi::MoveDetailsLight {
    /* .ply = */ ply,
    /* .role = */ role,
    /* .from = */ from,
    /* .to = */ to,
    /* .capture = */ capture,
    /* .is_castle = */ is_castle,
    /* .promotion = */ promotion,
    /* .is_en_passant = */ is_en_passant,
  };
}

inline MoveDetailsLight MoveDetailsLight::FromFFI(diplomat::capi::MoveDetailsLight c_struct) {
  return MoveDetailsLight {
    /* .ply = */ c_struct.ply,
    /* .role = */ c_struct.role,
    /* .from = */ c_struct.from,
    /* .to = */ c_struct.to,
    /* .capture = */ c_struct.capture,
    /* .is_castle = */ c_struct.is_castle,
    /* .promotion = */ c_struct.promotion,
    /* .is_en_passant = */ c_struct.is_en_passant,
  };
}


#endif // MoveDetailsLight_HPP
