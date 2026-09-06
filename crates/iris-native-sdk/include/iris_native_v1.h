#ifndef IRIS_NATIVE_V1_H
#define IRIS_NATIVE_V1_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef uint64_t IrisNativeHandle;
typedef struct IrisNativeHeaderV1 { uint32_t major, minor; size_t size; uint64_t features; } IrisNativeHeaderV1;
typedef struct IrisNativeSliceV1 { const uint8_t *data; size_t length; } IrisNativeSliceV1;
typedef struct IrisNativeScalarV1 { uint32_t kind; int64_t integer; } IrisNativeScalarV1;
typedef struct IrisNativeBlobInfoV1 { uint32_t kind; size_t length; } IrisNativeBlobInfoV1;
typedef struct IrisNativeResultV1 { size_t size; IrisNativeHandle value, context; } IrisNativeResultV1;
typedef struct IrisNativeHostV1 IrisNativeHostV1;
typedef int32_t (*IrisNativeFunctionV1)(const IrisNativeHostV1 *, void *, const IrisNativeHandle *, size_t, IrisNativeResultV1 *);
typedef int32_t (*IrisNativeCloseV1)(const IrisNativeHostV1 *, void *, uint64_t, IrisNativeResultV1 *);
typedef void (*IrisNativeDestroyV1)(uint64_t);
typedef struct IrisNativeResourceV1 { size_t size; IrisNativeCloseV1 close; IrisNativeDestroyV1 destroy; } IrisNativeResourceV1;
struct IrisNativeHostV1 {
    IrisNativeHeaderV1 header;
    int32_t (*scalar_read)(void *, IrisNativeHandle, IrisNativeScalarV1 *);
    int32_t (*scalar_create)(void *, IrisNativeScalarV1, IrisNativeHandle *);
    int32_t (*blob_info)(void *, IrisNativeHandle, IrisNativeBlobInfoV1 *);
    int32_t (*blob_read)(void *, IrisNativeHandle, uint8_t *, size_t);
    int32_t (*blob_create)(void *, uint32_t, IrisNativeSliceV1, IrisNativeHandle *);
    int32_t (*resource_create)(void *, size_t, uint64_t, IrisNativeHandle *);
    int32_t (*resource_read)(void *, IrisNativeHandle, size_t, uint64_t *);
    int32_t (*error_raise)(void *, IrisNativeSliceV1, IrisNativeSliceV1, int64_t, IrisNativeHandle *);
};
typedef struct IrisNativeModuleV1 {
    IrisNativeHeaderV1 header;
    size_t function_count, resource_count;
    int32_t (*function_at)(size_t, IrisNativeFunctionV1 *);
    int32_t (*resource_at)(size_t, IrisNativeResourceV1 *);
    uint8_t metadata_sha256[32];
} IrisNativeModuleV1;
int32_t iris_native_module_v1(IrisNativeModuleV1 *out);
#ifdef __cplusplus
}
#endif
#endif
