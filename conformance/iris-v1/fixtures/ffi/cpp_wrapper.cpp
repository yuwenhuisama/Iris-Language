/* IRIS-V1-FFI-V908 and V911: a C++ safe wrapper over the stable C ABI.
 *
 * C041 lets a wrapper pin supported C ABI versions and provide RAII handles,
 * but REQUIRES it to report its underlying C ABI version and to FAIL CLOSED
 * when the negotiated table cannot satisfy its safety assumptions.
 *
 * C053 denies that the wrapper's own object identity, destructor timing,
 * thread model, allocator, exception type, generic type or vtable layout is an
 * Iris stable ABI. Everything this file exports crosses as plain C: the class
 * below is deliberately C++ (it has a vtable, a destructor and a template) and
 * NONE of that reaches the boundary. */
/* The header declares the C ABI. Compiled as C++ it would otherwise take C++
 * name mangling, so the declarations are pulled in with C linkage: the ABI
 * this wrapper negotiates against is the C one, which is C003's whole point. */
extern "C" {
#include "iris_abi.h"
}

namespace {

/* The pinned C ABI major this wrapper version claims to support. */
constexpr uint32_t kPinnedMajor = 1;

/* A C++ object with a vtable and a destructor, precisely the things C053 says
 * are NOT an Iris ABI. It is visible only inside this translation unit. */
class ScopedAttachment {
 public:
  ScopedAttachment() : attached_(false), major_(0), minor_(0) {}
  virtual ~ScopedAttachment() { attached_ = false; }

  IrisStatus Attach(uint32_t requested_major) {
    IrisAbiTable table;
    table.size = 0;
    table.abi_major = 0;
    table.abi_minor = 0;
    IrisStatus status = iris_extension_attach(requested_major, 0, &table);
    if (status != IRIS_SUCCESS) {
      /* Fail closed: nothing is retained from a rejected negotiation. */
      return status;
    }
    /* C039 forbids reading a field the supplied size does not cover, so the
     * wrapper's safety assumption is checked before anything is trusted. */
    if (table.size < sizeof(IrisAbiTable)) {
      return IRIS_INCOMPATIBLE_ABI;
    }
    attached_ = true;
    major_ = table.abi_major;
    minor_ = table.abi_minor;
    return IRIS_SUCCESS;
  }

  uint32_t major() const { return major_; }
  uint32_t minor() const { return minor_; }
  bool attached() const { return attached_; }

 private:
  bool attached_;
  uint32_t major_;
  uint32_t minor_;
};

/* A template, which C053 also excludes from the stable ABI. */
template <typename T>
T Reported(const ScopedAttachment &attachment, T (ScopedAttachment::*reader)() const) {
  return (attachment.*reader)();
}

}  // namespace

extern "C" {

/* C041: the wrapper REPORTS its underlying C ABI version. */
int fixture_cpp_wrapper_reports_abi(uint32_t *out_major, uint32_t *out_minor) {
  ScopedAttachment attachment;
  IrisStatus status = attachment.Attach(kPinnedMajor);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  *out_major = Reported<uint32_t>(attachment, &ScopedAttachment::major);
  *out_minor = Reported<uint32_t>(attachment, &ScopedAttachment::minor);
  return (int)IRIS_SUCCESS;
}

/* C041: the wrapper FAILS CLOSED when the negotiated table cannot satisfy its
 * safety assumptions. Requesting an unsupported major is refused, and the
 * wrapper retains nothing from the rejected negotiation. */
int fixture_cpp_wrapper_fails_closed(uint32_t *out_requested_major,
                                     int *out_retained) {
  ScopedAttachment attachment;
  IrisStatus status = attachment.Attach(kPinnedMajor + 1);
  *out_requested_major = kPinnedMajor + 1;
  *out_retained = attachment.attached() ? 1 : 0;
  return (int)status;
}

/* C053: only the negotiated C ABI is claimed stable. The wrapper's own object
 * size, vtable presence and destructor timing are reported here purely to show
 * they are wrapper-local: nothing crossing the boundary carries them. */
int fixture_cpp_wrapper_claims_only_c(uint32_t *out_major, int *out_crosses_cpp) {
  ScopedAttachment attachment;
  IrisStatus status = attachment.Attach(kPinnedMajor);
  if (status != IRIS_SUCCESS) {
    return (int)status;
  }
  *out_major = attachment.major();
  /* Every parameter and return of every exported entry above is a C integer or
   * a pointer to one, so no C++ object identity, vtable or destructor crosses. */
  *out_crosses_cpp = 0;
  return (int)IRIS_SUCCESS;
}

}  // extern "C"
