#ifndef MoveDetailsExtIterator_HPP
#define MoveDetailsExtIterator_HPP

#include "MoveDetailsExtIterator.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "DecodeError.hpp"
#include "MoveDetailsExtended.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {
    
    typedef struct MoveDetailsExtIterator_next_result {union {diplomat::capi::MoveDetailsExtended ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsExtIterator_next_result;
    MoveDetailsExtIterator_next_result MoveDetailsExtIterator_next(diplomat::capi::MoveDetailsExtIterator* self);
    
    typedef struct MoveDetailsExtIterator_nth_result {union {diplomat::capi::MoveDetailsExtended ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsExtIterator_nth_result;
    MoveDetailsExtIterator_nth_result MoveDetailsExtIterator_nth(diplomat::capi::MoveDetailsExtIterator* self, int16_t n);
    
    
    void MoveDetailsExtIterator_destroy(MoveDetailsExtIterator* self);
    
    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<MoveDetailsExtended, DecodeError> MoveDetailsExtIterator::next() {
  auto result = diplomat::capi::MoveDetailsExtIterator_next(this->AsFFI());
  return result.is_ok ? diplomat::result<MoveDetailsExtended, DecodeError>(diplomat::Ok<MoveDetailsExtended>(MoveDetailsExtended::FromFFI(result.ok))) : diplomat::result<MoveDetailsExtended, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline diplomat::result<MoveDetailsExtended, DecodeError> MoveDetailsExtIterator::nth(int16_t n) {
  auto result = diplomat::capi::MoveDetailsExtIterator_nth(this->AsFFI(),
    n);
  return result.is_ok ? diplomat::result<MoveDetailsExtended, DecodeError>(diplomat::Ok<MoveDetailsExtended>(MoveDetailsExtended::FromFFI(result.ok))) : diplomat::result<MoveDetailsExtended, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline const diplomat::capi::MoveDetailsExtIterator* MoveDetailsExtIterator::AsFFI() const {
  return reinterpret_cast<const diplomat::capi::MoveDetailsExtIterator*>(this);
}

inline diplomat::capi::MoveDetailsExtIterator* MoveDetailsExtIterator::AsFFI() {
  return reinterpret_cast<diplomat::capi::MoveDetailsExtIterator*>(this);
}

inline const MoveDetailsExtIterator* MoveDetailsExtIterator::FromFFI(const diplomat::capi::MoveDetailsExtIterator* ptr) {
  return reinterpret_cast<const MoveDetailsExtIterator*>(ptr);
}

inline MoveDetailsExtIterator* MoveDetailsExtIterator::FromFFI(diplomat::capi::MoveDetailsExtIterator* ptr) {
  return reinterpret_cast<MoveDetailsExtIterator*>(ptr);
}

inline void MoveDetailsExtIterator::operator delete(void* ptr) {
  diplomat::capi::MoveDetailsExtIterator_destroy(reinterpret_cast<diplomat::capi::MoveDetailsExtIterator*>(ptr));
}


#endif // MoveDetailsExtIterator_HPP
