#ifndef _H_IRISMUTEX_
#define _H_IRISMUTEX_

#include "IrisDevHeader.h"
#include "IrisMutexTag.h"

class IrisMutex : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue Lock(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {

		if (!pContextEnvironment->GetClosureBlock()) {
			IrisDev_GroanIrregularWithString("Method of lock of a Mutex object need a block to run.");
			return IrisDev_Nil();
		}

		//ivObj.GetIrisObject()->SetPermanent(true);
		auto pMutex = IrisDev_GetNativePointer<IrisMutexTag*>(ivObj);
		pMutex->Lock();
		//IrisGC::SetGCFlag(false);
		pContextEnvironment->GetClosureBlock()->Excute(nullptr);
		//IrisGC::SetGCFlag(true);
		pMutex->Unlock();
		//ivObj.GetIrisObject()->SetPermanent(false);

		return IrisDev_Nil();
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Mutex";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
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
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "lock", Lock, 0, false);
	}


	IrisMutex();
	~IrisMutex();
};

#endif