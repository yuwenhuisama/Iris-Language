/* IRIS-V1-FFI-V009: a native artifact whose digest mismatches its metadata.
 *
 * C023 makes runtime native load verify the loaded artifact against the
 * selected metadata, and V009 aborts PACKAGE OR EXTENSION load before binding.
 * V064 covers the package path; this is the extension path, which negotiates a
 * C ABI table rather than resolving a package.
 *
 * The table is pre-filled with a sentinel so a refused load can be seen to
 * leave it untouched: receiving a negotiated table would mean an unverified
 * extension had already been handed authority. */
#include "iris_abi.h"

int fixture_extension_digest_mismatch(uint32_t *out_major) {
  IrisAbiTable table;
  table.size = 0;
  table.abi_major = 99;

  IrisStatus status = iris_extension_load(
      "0000000000000000000000000000000000000000000000000000000000000011",
      "a192491d7d9b183ab3857f58a71a6f69dd31e6554917696bbd32b0f246147608", 1, 0,
      &table);

  *out_major = table.abi_major;
  return (int)status;
}

/* The same load with a digest that MATCHES must attach, which is what shows
 * the refusal above comes from verification and not from the path being
 * broken outright. */
int fixture_extension_digest_matches(uint32_t *out_major) {
  IrisAbiTable table;
  table.size = 0;
  table.abi_major = 99;

  IrisStatus status = iris_extension_load(
      "a192491d7d9b183ab3857f58a71a6f69dd31e6554917696bbd32b0f246147608",
      "a192491d7d9b183ab3857f58a71a6f69dd31e6554917696bbd32b0f246147608", 1, 0,
      &table);

  *out_major = table.abi_major;
  return (int)status;
}
