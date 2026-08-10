/* IRIS-V1-FFI-V005: a worker thread posts copied completion data and a token,
 * and the RUNTIME thread is what turns it into a completed value.
 *
 * C013 permits an external thread to post COPIED data only, never a handle,
 * which is why the payload here is an integer and the worker never touches the
 * handle table. C014 makes this queue the only cross-thread entry, and C037
 * ties the delivered value to the token that authorized it: draining reports
 * the token back so the completion can be matched to its own request rather
 * than merely observed to have arrived. */
#include "iris_abi.h"

int fixture_worker_completes(uint64_t token, int64_t value) {
  return (int)iris_post_completion(token, value);
}

int fixture_runtime_takes_completion(uint64_t *out_token, int64_t *out_value,
                                     uint32_t *out_count) {
  return (int)iris_drain_first_completion(out_token, out_value, out_count);
}
