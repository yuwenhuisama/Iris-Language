#ifndef _H_IRISMUTEX_
#define _H_IRISMUTEX_

#include "IrisDevHeader.h"
#include "IrisMutexTag.h"

class IrisMutex : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Lock(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Mutex";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisMutexTag);
	}

	void* NativeAlloc() {
		return new IrisMutexTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisMutexTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "lock", Lock, 0, false);
	}


	IrisMutex();
	~IrisMutex();
};

#endif