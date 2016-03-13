#ifndef _H_IRISCONDITIONVARIABLE_
#define _H_IRISCONDITIONVARIABLE_

#include "IrisDevHeader.h"

#include "IrisMutex.h"
#include "IrisConditionVariableTag.h"

class IrisConditionVariable : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {

		if (!IrisDev_CheckClass((IrisValue&)ivsValues->GetValue(0), "Mutex")) {
			IrisDev_GroanIrregularWithString("Invalid Parameter.");
			return IrisDev_Nil();
		}

		auto pMutex = &IrisDev_GetNativePointer<IrisMutexTag*>((IrisValue&)ivsValues->GetValue(0))->GetMutex();
		auto pConditionVariable = IrisDev_GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->Initialize(pMutex);
		ivObj.GetIrisObject()->AddInstanceValue("@mutex", (IrisValue&)ivsValues->GetValue(0));
		return ivObj;
	}

	static IrisValue NotifyAll(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDev_GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->NotifyAll();
		return IrisDev_Nil();
	}

	static IrisValue NotifyOne(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDev_GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->NotifyOne();
		return IrisDev_Nil();
	}

	static IrisValue Wait(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDev_GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->Wait();
		return IrisDev_Nil();
	}

public:
	
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "ConditionVariable";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisConditionVariableTag);
	}

	void* NativeAlloc() {
		return new IrisConditionVariableTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisConditionVariableTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDev_AddInstanceMethod(this, "notify_one", NotifyOne, 0, false);
		IrisDev_AddInstanceMethod(this, "notify_all", NotifyAll, 0, false);
		IrisDev_AddInstanceMethod(this, "wait", Wait, 0, false);
	}

	IrisConditionVariable();
	~IrisConditionVariable();
};

#endif