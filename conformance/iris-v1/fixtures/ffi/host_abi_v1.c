/* IRIS-V1-FFI-V060: attach through the Host table and report the negotiated
 * versions. The extension receives ONLY this C table: no Rust, C++,
 * object-layout, vtable or allocator entry is reachable from here. */
#include "iris_abi.h"

int fixture_host_abi_v1(uint32_t *out_major, uint32_t *out_minor) {
  IrisAbiTable table;
  IrisStatus status = iris_extension_attach(1, 0, &table);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  /* C039 forbids reading a field the supplied size does not cover. */
  if (table.size < sizeof(IrisAbiTable)) {
    return (int)IRIS_INCOMPATIBLE_ABI;
  }
  *out_major = table.abi_major;
  *out_minor = table.abi_minor;
  return (int)IRIS_SUCCESS;
}
