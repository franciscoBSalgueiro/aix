#ifndef MoveDetailsFullIterator_HPP
#define MoveDetailsFullIterator_HPP

#include "MoveDetailsFullIterator.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "DecodeError.hpp"
#include "MoveDetailsFull.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {
    
    typedef struct MoveDetailsFullIterator_next_result {union {diplomat::capi::MoveDetailsFull ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsFullIterator_next_result;
    MoveDetailsFullIterator_next_result MoveDetailsFullIterator_next(diplomat::capi::MoveDetailsFullIterator* self);
    
    typedef struct MoveDetailsFullIterator_nth_result {union {diplomat::capi::MoveDetailsFull ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsFullIterator_nth_result;
    MoveDetailsFullIterator_nth_result MoveDetailsFullIterator_nth(diplomat::capi::MoveDetailsFullIterator* self, int16_t n);
    
    
    void MoveDetailsFullIterator_destroy(MoveDetailsFullIterator* self);
    
    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<MoveDetailsFull, DecodeError> MoveDetailsFullIterator::next() {
  auto result = diplomat::capi::MoveDetailsFullIterator_next(this->AsFFI());
  return result.is_ok ? diplomat::result<MoveDetailsFull, DecodeError>(diplomat::Ok<MoveDetailsFull>(MoveDetailsFull::FromFFI(result.ok))) : diplomat::result<MoveDetailsFull, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline diplomat::result<MoveDetailsFull, DecodeError> MoveDetailsFullIterator::nth(int16_t n) {
  auto result = diplomat::capi::MoveDetailsFullIterator_nth(this->AsFFI(),
    n);
  return result.is_ok ? diplomat::result<MoveDetailsFull, DecodeError>(diplomat::Ok<MoveDetailsFull>(MoveDetailsFull::FromFFI(result.ok))) : diplomat::result<MoveDetailsFull, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline const diplomat::capi::MoveDetailsFullIterator* MoveDetailsFullIterator::AsFFI() const {
  return reinterpret_cast<const diplomat::capi::MoveDetailsFullIterator*>(this);
}

inline diplomat::capi::MoveDetailsFullIterator* MoveDetailsFullIterator::AsFFI() {
  return reinterpret_cast<diplomat::capi::MoveDetailsFullIterator*>(this);
}

inline const MoveDetailsFullIterator* MoveDetailsFullIterator::FromFFI(const diplomat::capi::MoveDetailsFullIterator* ptr) {
  return reinterpret_cast<const MoveDetailsFullIterator*>(ptr);
}

inline MoveDetailsFullIterator* MoveDetailsFullIterator::FromFFI(diplomat::capi::MoveDetailsFullIterator* ptr) {
  return reinterpret_cast<MoveDetailsFullIterator*>(ptr);
}

inline void MoveDetailsFullIterator::operator delete(void* ptr) {
  diplomat::capi::MoveDetailsFullIterator_destroy(reinterpret_cast<diplomat::capi::MoveDetailsFullIterator*>(ptr));
}


#endif // MoveDetailsFullIterator_HPP
