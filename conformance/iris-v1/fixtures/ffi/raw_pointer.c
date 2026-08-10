/* IRIS-V1-FFI-V001: a native extension tries to pass a raw managed pointer
 * across the C ABI. C007 forbids a raw managed pointer, GC address, vtable
 * address or interior pointer from crossing, so the only thing this fixture
 * CAN do is fabricate a handle out of an address and hand it over.
 *
 * Note first what the header already proves: no exported call hands an address
 * out, so a fixture must invent one to attempt the smuggling at all.
 *
 * Two forgeries are attempted, because the naive one is refused for a reason
 * that is partly luck. A stack address has zero high bits, so it carries
 * runtime tag 0 and is rejected by tag validation before any lookup. A forger
 * who knows the layout stamps the correct tag and puts the address in the slot
 * field, which reaches the table lookup instead. Both must be refused, and
 * neither may dereference: the pointee is 41, so observing 41 through either
 * path would prove the boundary leaked. */
#include "iris_abi.h"

int fixture_raw_pointer_handle(int64_t *out_value, int32_t *out_touched,
                               int32_t *out_tagged_status) {
  int64_t local = 41;
  *out_touched = 0;

  /* Forgery one: reinterpret the address itself as a handle. */
  IrisHandle forged = (IrisHandle)(uintptr_t)&local;
  int64_t naive_value = 0;
  IrisStatus status = iris_handle_get_int(forged, &naive_value);

  /* Forgery two: the same address smuggled into the slot field under the
   * runtime tag the table actually uses, so tag validation cannot be what
   * refuses it. */
  IrisHandle tagged =
      ((IrisHandle)1u << 48) | (IrisHandle)(uint32_t)(uintptr_t)&local;
  int64_t tagged_value = 0;
  *out_tagged_status = (int32_t)iris_handle_get_int(tagged, &tagged_value);

  *out_value = naive_value;
  *out_touched = (naive_value == 41 || tagged_value == 41) ? 1 : 0;
  return (int)status;
}
