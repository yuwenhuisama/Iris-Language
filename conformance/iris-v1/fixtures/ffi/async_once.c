/* IRIS-V1-FFI-V066: post copied integer 9 twice with ONE completion token.
 * C037 makes the first completion stand, so the second post is refused and the
 * completion count stays 1. */
#include "iris_abi.h"

int fixture_post_twice(uint64_t token, int *out_second_status,
                       uint32_t *out_count, int64_t *out_value) {
  IrisStatus first = iris_post_completion(token, 9);
  if (first != IRIS_SUCCESS) {
    return (int)first;
  }
  *out_second_status = (int)iris_post_completion(token, 9);
  return (int)iris_drain_completions(out_count, out_value);
}
