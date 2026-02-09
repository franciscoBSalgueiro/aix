#ifndef MoveDetailsFullIterator_D_HPP
#define MoveDetailsFullIterator_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <optional>
#include "diplomat_runtime.hpp"

struct MoveDetailsFull;
class DecodeError;


namespace diplomat {
namespace capi {
    struct MoveDetailsFullIterator;
} // namespace capi
} // namespace

class MoveDetailsFullIterator {
public:

  inline diplomat::result<MoveDetailsFull, DecodeError> next();

  inline diplomat::result<MoveDetailsFull, DecodeError> nth(int16_t n);

  inline const diplomat::capi::MoveDetailsFullIterator* AsFFI() const;
  inline diplomat::capi::MoveDetailsFullIterator* AsFFI();
  inline static const MoveDetailsFullIterator* FromFFI(const diplomat::capi::MoveDetailsFullIterator* ptr);
  inline static MoveDetailsFullIterator* FromFFI(diplomat::capi::MoveDetailsFullIterator* ptr);
  inline static void operator delete(void* ptr);
private:
  MoveDetailsFullIterator() = delete;
  MoveDetailsFullIterator(const MoveDetailsFullIterator&) = delete;
  MoveDetailsFullIterator(MoveDetailsFullIterator&&) noexcept = delete;
  MoveDetailsFullIterator operator=(const MoveDetailsFullIterator&) = delete;
  MoveDetailsFullIterator operator=(MoveDetailsFullIterator&&) noexcept = delete;
  static void operator delete[](void*, size_t) = delete;
};


#endif // MoveDetailsFullIterator_D_HPP
