/* IRIS-V1-FFI-V063: native code raises an Iris value through the Host ABI.
 * C017 forbids a status-only answer when the call raised, so the context
 * handle is filled as well; C019 forbids a foreign unwind from crossing. */
#include "iris_abi.h"

int fixture_raise_marker(IrisHandle *out_context, int64_t *out_marker) {
  IrisStatus status = iris_raise_marker(41, out_context);
  if (status != IRIS_RAISED) {
    return (int)status;
  }
  /* The raised value stays an ordinary Iris value reachable through the
   * context handle, not a string baked into the status. */
  if (iris_handle_get_int(*out_context, out_marker) != IRIS_SUCCESS) {
    return (int)IRIS_INVALID_HANDLE;
  }
  return (int)IRIS_RAISED;
}
