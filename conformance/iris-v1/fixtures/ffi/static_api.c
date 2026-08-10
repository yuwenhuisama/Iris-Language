/* IRIS-V1-FFI-V064: the artifact a native package metadata record points at.
 *
 * C023 verifies the loaded artifact against the resolved metadata BEFORE
 * binding, so this file exists to be hashed and refused, not to be called. It
 * exports a working entry on purpose: the load must fail on the digest, not
 * because there was nothing here to bind.
 *
 * The call counter is what makes "no native code is called" an OBSERVATION
 * rather than a claim. A refused load must leave it at zero. */
#include "iris_abi.h"

static int32_t call_count = 0;

int64_t fixture_static_api_answer(void) {
  call_count += 1;
  return 41;
}

int32_t fixture_static_api_call_count(void) { return call_count; }
