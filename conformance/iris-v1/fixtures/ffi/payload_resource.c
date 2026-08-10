/* IRIS-V1-FFI-V065 and V012: a native-backed Closeable resource closed twice.
 *
 * C027 gives the RUNTIME control of payload storage, so this extension only
 * registers a descriptor {size 8, alignment 8, trace none, cleanup no_raise}
 * and never allocates the payload itself. C030 makes deterministic release
 * explicit and idempotent, so closing twice must release exactly once.
 *
 * The release count is the real observation: two calls that both merely
 * "succeed" would be indistinguishable from a double release without it. */
#include "iris_abi.h"

int fixture_payload_close_twice(int *out_second, uint32_t *out_releases) {
  uint32_t diagnostic = 0;
  IrisStatus registered =
      iris_payload_register(8, 8, 0, 1, &diagnostic);
  if (registered != IRIS_SUCCESS) {
    return (int)registered;
  }

  IrisStatus first = iris_payload_close();
  *out_second = (int)iris_payload_close();

  /* C030 permits final cleanup to release as a last resort, so running it
   * after an explicit close must NOT release a second time. */
  iris_payload_final_cleanup();
  *out_releases = iris_payload_release_count();
  return (int)first;
}

/* IRIS-V1-FFI-V013: a descriptor whose final cleanup claims it may raise into
 * Iris. C029 makes cleanup failures runtime diagnostics rather than catchable
 * language results, so this is refused at REGISTRATION: the runtime never
 * accepts the descriptor and then contains a raise at drop time. */
int fixture_payload_cleanup_may_raise(uint32_t *out_diagnostic,
                                      uint32_t *out_releases) {
  IrisStatus status = iris_payload_register(8, 8, 1, 1, out_diagnostic);
  /* A refused descriptor owns no storage, so there is nothing to release. */
  *out_releases = iris_payload_release_count();
  return (int)status;
}
