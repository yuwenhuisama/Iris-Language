/* IRIS-V1-FFI-V067: an extension requests ABI major 2 from a runtime that
 * implements major 1. C039 rejects a major mismatch outright, so no table is
 * published and no extension entry runs. */
#include "iris_abi.h"

int fixture_negotiate_v2(uint32_t *out_major) {
  IrisAbiTable table;
  table.size = 0;
  table.abi_major = 0;
  IrisStatus status = iris_extension_attach(2, 0, &table);
  /* No table is published on rejection, so the record stays untouched. */
  *out_major = table.abi_major;
  return (int)status;
}
