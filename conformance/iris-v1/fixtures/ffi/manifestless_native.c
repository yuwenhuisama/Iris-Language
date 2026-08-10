/* IRIS-V1-FFI-V059: a native artifact whose metadata omits package identity.
 *
 * C022 makes the compiler import native metadata as static API ONLY after
 * validating it is complete, so this artifact must never be bound. As with
 * V064 the entry works on purpose: the refusal has to come from the missing
 * identity, not from an artifact that had nothing to offer.
 *
 * The call counter makes "reflection metadata is not published" observable
 * rather than asserted. A refused load must leave it at zero. */
#include "iris_abi.h"

static int32_t manifestless_calls = 0;

int64_t fixture_manifestless_entry(void) {
  manifestless_calls += 1;
  return 7;
}

int32_t fixture_manifestless_call_count(void) { return manifestless_calls; }
