#ifndef MoveDetailsExtIterator_D_HPP
#define MoveDetailsExtIterator_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"

struct MoveDetailsExtended;
class DecodeError;


namespace diplomat {
namespace capi {
    struct MoveDetailsExtIterator;
} // namespace capi
} // namespace

class MoveDetailsExtIterator {
public:

  inline diplomat::result<MoveDetailsExtended, DecodeError> next();

  inline diplomat::result<MoveDetailsExtended, DecodeError> nth(int16_t n);

  inline const diplomat::capi::MoveDetailsExtIterator* AsFFI() const;
  inline diplomat::capi::MoveDetailsExtIterator* AsFFI();
  inline static const MoveDetailsExtIterator* FromFFI(const diplomat::capi::MoveDetailsExtIterator* ptr);
  inline static MoveDetailsExtIterator* FromFFI(diplomat::capi::MoveDetailsExtIterator* ptr);
  inline static void operator delete(void* ptr);
private:
  MoveDetailsExtIterator() = delete;
  MoveDetailsExtIterator(const MoveDetailsExtIterator&) = delete;
  MoveDetailsExtIterator(MoveDetailsExtIterator&&) noexcept = delete;
  MoveDetailsExtIterator operator=(const MoveDetailsExtIterator&) = delete;
  MoveDetailsExtIterator operator=(MoveDetailsExtIterator&&) noexcept = delete;
  static void operator delete[](void*, size_t) = delete;
};


#endif // MoveDetailsExtIterator_D_HPP
