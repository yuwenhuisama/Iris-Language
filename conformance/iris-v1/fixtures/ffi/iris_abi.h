/* The Iris v1 stable C ABI.
 *
 * IRIS-V1-FFI-C003 makes this header the binary compatibility identity. It
 * declares only C types: no C++ class, no Rust type, no vtable, no allocator,
 * and no object layout. IRIS-V1-FFI-C042 lists promising such a layout as
 * Prohibited, which is why every value crossing here is an integer handle.
 */
#ifndef IRIS_ABI_H
#define IRIS_ABI_H

#include <stdint.h>

/* IRIS-V1-FFI-C017 status codes. Discriminants are fixed for ABI major 1;
 * C042 makes changing the meaning of an existing status major-breaking. */
typedef enum {
  IRIS_SUCCESS = 0,
  IRIS_INVALID_HANDLE = 1,
  IRIS_INVALID_RUNTIME = 2,
  IRIS_THREAD_AFFINITY = 3,
  IRIS_RAISED = 4,
  IRIS_DUPLICATE_COMPLETION = 5,
  IRIS_INCOMPATIBLE_ABI = 6,
  IRIS_INVALID_ARGUMENT = 7,
  IRIS_INVALID_BOUNDARY = 8
} IrisStatus;

/* IRIS-V1-FFI-C007: an opaque handle, never an address. The bits are a runtime
 * tag, a slot and a generation; C009 forbids treating them as stable identity,
 * persisting them, or assuming a slot is never reused. */
typedef uint64_t IrisHandle;

/* IRIS-V1-FFI-C038 size-tagged negotiation record. `size` comes first so a
 * reader can tell which fields the producer actually filled in. */
typedef struct {
  uint32_t size;
  uint32_t abi_major;
  uint32_t abi_minor;
  uint32_t features;
} IrisAbiTable;

IrisStatus iris_extension_attach(uint32_t requested_major,
                                 uint32_t minimum_minor,
                                 IrisAbiTable *out_table);
IrisStatus iris_int_create(int64_t value, IrisHandle *out_handle);
IrisStatus iris_handle_get_int(IrisHandle handle, int64_t *out_value);
IrisStatus iris_handle_release(IrisHandle handle);
void iris_runtime_reset(void);
void iris_bridge_reset(void);

/* IRIS-V1-FFI-C013: an external thread posts COPIED data only, never a handle.
 * C037 makes the first completion for a token stand. */
IrisStatus iris_post_completion(uint64_t token, int64_t value);
IrisStatus iris_drain_completions(uint32_t *out_count, int64_t *out_first);

/* IRIS-V1-FFI-C037: the runtime thread learns WHICH token a value belongs to,
 * so a completion can be matched to its own request. */
IrisStatus iris_drain_first_completion(uint64_t *out_token, int64_t *out_value,
                                       uint32_t *out_count);

/* IRIS-V1-FFI-C017/C018: a raise answers a status AND fills a context handle.
 * A string-only error channel is not conforming. */
IrisStatus iris_raise_marker(int64_t marker, IrisHandle *out_context);

/* IRIS-V1-FFI-C019: a panic beneath the boundary becomes a returned status.
 * No unwinding, exception or long jump reaches this caller. */
IrisStatus iris_call_panicking(int64_t *out_value);

#endif /* IRIS_ABI_H */
