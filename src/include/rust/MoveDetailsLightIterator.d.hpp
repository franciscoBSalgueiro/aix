#ifndef MoveDetailsLightIterator_D_HPP
#define MoveDetailsLightIterator_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"

struct MoveDetailsLight;
class DecodeError;


namespace diplomat {
namespace capi {
    struct MoveDetailsLightIterator;
} // namespace capi
} // namespace

class MoveDetailsLightIterator {
public:

  inline diplomat::result<MoveDetailsLight, DecodeError> next();

  inline diplomat::result<MoveDetailsLight, DecodeError> nth(int16_t n);

  inline const diplomat::capi::MoveDetailsLightIterator* AsFFI() const;
  inline diplomat::capi::MoveDetailsLightIterator* AsFFI();
  inline static const MoveDetailsLightIterator* FromFFI(const diplomat::capi::MoveDetailsLightIterator* ptr);
  inline static MoveDetailsLightIterator* FromFFI(diplomat::capi::MoveDetailsLightIterator* ptr);
  inline static void operator delete(void* ptr);
private:
  MoveDetailsLightIterator() = delete;
  MoveDetailsLightIterator(const MoveDetailsLightIterator&) = delete;
  MoveDetailsLightIterator(MoveDetailsLightIterator&&) noexcept = delete;
  MoveDetailsLightIterator operator=(const MoveDetailsLightIterator&) = delete;
  MoveDetailsLightIterator operator=(MoveDetailsLightIterator&&) noexcept = delete;
  static void operator delete[](void*, size_t) = delete;
};


#endif // MoveDetailsLightIterator_D_HPP
