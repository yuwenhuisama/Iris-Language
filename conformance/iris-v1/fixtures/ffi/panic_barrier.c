/* IRIS-V1-FFI-V008: an unwinding failure beneath the boundary is caught BEFORE
 * it reaches C. C019 forbids a Rust panic, C++ exception, SEH exception, host
 * unwind or long jump from crossing the C ABI, so this C frame must observe an
 * ordinary returned status and keep running normally afterwards. */
#include "iris_abi.h"

int fixture_panic_does_not_cross(int64_t *out_value, int32_t *out_resumed) {
  *out_resumed = 0;
  IrisStatus status = iris_call_panicking(out_value);
  /* Reaching this line at all is the observation: the C frame was not
   * unwound through, so ordinary execution continues after the failure. */
  *out_resumed = 1;
  return (int)status;
}
