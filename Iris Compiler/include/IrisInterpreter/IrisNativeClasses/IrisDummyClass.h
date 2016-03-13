#ifndef _H_IRISDUMMYCLASS_
#define _H_IRISDUMMYCLASS_

#include "IrisInterfaces/IIrisClass.h"
class IrisDummyClass :
	public IIrisClass
{
public:

	int GetTrustteeSize(void* pNativePointer) {
		return 0;
	}

	void Mark(void* pNativeObjectPointer) {}

	void* NativeAlloc() {
		return nullptr;
	}

	void NativeFree(void* pNativePointer) {
	}

	void NativeClassDefine() {
	}

	const char* NativeClassNameDefine() const {
		return nullptr;
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	IrisDummyClass();
	~IrisDummyClass();
};

#endif