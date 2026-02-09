#ifndef MoveDetailsLightIterator_HPP
#define MoveDetailsLightIterator_HPP

#include "MoveDetailsLightIterator.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "DecodeError.hpp"
#include "MoveDetailsLight.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {
    
    typedef struct MoveDetailsLightIterator_next_result {union {diplomat::capi::MoveDetailsLight ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsLightIterator_next_result;
    MoveDetailsLightIterator_next_result MoveDetailsLightIterator_next(diplomat::capi::MoveDetailsLightIterator* self);
    
    typedef struct MoveDetailsLightIterator_nth_result {union {diplomat::capi::MoveDetailsLight ok; diplomat::capi::DecodeError err;}; bool is_ok;} MoveDetailsLightIterator_nth_result;
    MoveDetailsLightIterator_nth_result MoveDetailsLightIterator_nth(diplomat::capi::MoveDetailsLightIterator* self, int16_t n);
    
    
    void MoveDetailsLightIterator_destroy(MoveDetailsLightIterator* self);
    
    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<MoveDetailsLight, DecodeError> MoveDetailsLightIterator::next() {
  auto result = diplomat::capi::MoveDetailsLightIterator_next(this->AsFFI());
  return result.is_ok ? diplomat::result<MoveDetailsLight, DecodeError>(diplomat::Ok<MoveDetailsLight>(MoveDetailsLight::FromFFI(result.ok))) : diplomat::result<MoveDetailsLight, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline diplomat::result<MoveDetailsLight, DecodeError> MoveDetailsLightIterator::nth(int16_t n) {
  auto result = diplomat::capi::MoveDetailsLightIterator_nth(this->AsFFI(),
    n);
  return result.is_ok ? diplomat::result<MoveDetailsLight, DecodeError>(diplomat::Ok<MoveDetailsLight>(MoveDetailsLight::FromFFI(result.ok))) : diplomat::result<MoveDetailsLight, DecodeError>(diplomat::Err<DecodeError>(DecodeError::FromFFI(result.err)));
}

inline const diplomat::capi::MoveDetailsLightIterator* MoveDetailsLightIterator::AsFFI() const {
  return reinterpret_cast<const diplomat::capi::MoveDetailsLightIterator*>(this);
}

inline diplomat::capi::MoveDetailsLightIterator* MoveDetailsLightIterator::AsFFI() {
  return reinterpret_cast<diplomat::capi::MoveDetailsLightIterator*>(this);
}

inline const MoveDetailsLightIterator* MoveDetailsLightIterator::FromFFI(const diplomat::capi::MoveDetailsLightIterator* ptr) {
  return reinterpret_cast<const MoveDetailsLightIterator*>(ptr);
}

inline MoveDetailsLightIterator* MoveDetailsLightIterator::FromFFI(diplomat::capi::MoveDetailsLightIterator* ptr) {
  return reinterpret_cast<MoveDetailsLightIterator*>(ptr);
}

inline void MoveDetailsLightIterator::operator delete(void* ptr) {
  diplomat::capi::MoveDetailsLightIterator_destroy(reinterpret_cast<diplomat::capi::MoveDetailsLightIterator*>(ptr));
}


#endif // MoveDetailsLightIterator_HPP
