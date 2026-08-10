/* IRIS-V1-FFI-V061: create integer 41, retain it, read it, release it, then
 * read again. C008 keeps the value rooted while the handle lives; C009 makes
 * the released handle detectably stale rather than denoting a new occupant. */
#include "iris_abi.h"

int fixture_rooted_handle(int64_t *out_before, int *out_after_status) {
  IrisHandle handle;
  IrisStatus status = iris_int_create(41, &handle);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  status = iris_handle_get_int(handle, out_before);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  status = iris_handle_release(handle);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  /* After release the second read must report invalid-handle, and no value is
   * written through the out parameter. */
  int64_t discarded = 0;
  *out_after_status = (int)iris_handle_get_int(handle, &discarded);
  return (int)IRIS_SUCCESS;
}
