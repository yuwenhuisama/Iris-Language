/* IRIS-V1-FFI-V011: a native payload traces managed handle roots only.
 *
 * C028 limits trace logic to REPORTING managed handles or roots. It may not
 * dereference moved objects, retain raw managed pointers, create Iris values,
 * call arbitrary Iris Methods, raise, block on IO, or touch the heap from a
 * worker. This fixture can therefore only declare a root and ask the runtime
 * to read it back: there is no trace callback for an extension to abuse.
 *
 * Reporting a root must also KEEP THE TARGET ALIVE, so the live count is the
 * load-bearing observation rather than the reported count. */
#include "iris_abi.h"

int fixture_payload_traces_root(int64_t *out_value, uint32_t *out_reported,
                                uint32_t *out_live) {
  IrisHandle root = 0;
  IrisStatus created = iris_int_create(41, &root);
  if (created != IRIS_SUCCESS) {
    return (int)created;
  }

  uint32_t diagnostic = 0;
  IrisStatus registered = iris_payload_register(8, 8, 0, 0, &diagnostic);
  if (registered != IRIS_SUCCESS) {
    return (int)registered;
  }

  IrisStatus added = iris_payload_add_root(root);
  if (added != IRIS_SUCCESS) {
    return (int)added;
  }

  IrisStatus traced = iris_payload_trace(out_reported, out_live);
  /* The reported root still resolves to its value, which is what "referenced
   * managed values remain alive" means in observable terms. */
  iris_handle_get_int(root, out_value);
  return (int)traced;
}

/* A root must name a live managed value. A released handle is refused at
 * declaration rather than being handed to the collector later. */
int fixture_payload_rejects_stale_root(void) {
  IrisHandle root = 0;
  iris_int_create(7, &root);
  uint32_t diagnostic = 0;
  iris_payload_register(8, 8, 0, 0, &diagnostic);
  iris_handle_release(root);
  return (int)iris_payload_add_root(root);
}
