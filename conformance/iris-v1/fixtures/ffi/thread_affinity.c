/* IRIS-V1-FFI-V062: a worker reads through a handle and then posts copied data.
 * C012 refuses the read with a thread-affinity status rather than racing the
 * heap; C013 accepts the copied post, which the runtime thread consumes. */
#include "iris_abi.h"

int fixture_worker_reads_handle(IrisHandle handle, int64_t *out_value) {
  /* The read must be refused, and nothing may be written through the out
   * parameter when it is. */
  return (int)iris_handle_get_int(handle, out_value);
}

int fixture_worker_posts(uint64_t token, int64_t value) {
  return (int)iris_post_completion(token, value);
}
