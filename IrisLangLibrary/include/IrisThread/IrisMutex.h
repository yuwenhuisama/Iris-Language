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
			IrisDevUtil::GroanIrregularWithString("Method of lock of a Mutex object need a block to run.");
			return IrisDevUtil::Nil();
		}

		//ivObj.GetIrisObject()->SetPermanent(true);
		auto pMutex = IrisDevUtil::GetNativePointer<IrisMutexTag*>(ivObj);
		pMutex->Lock();
		//IrisGC::SetGCFlag(false);
		pContextEnvironment->GetClosureBlock()->Excute(nullptr);
		//IrisGC::SetGCFlag(true);
		pMutex->Unlock();
		//ivObj.GetIrisObject()->SetPermanent(false);

		return IrisDevUtil::Nil();
	}

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