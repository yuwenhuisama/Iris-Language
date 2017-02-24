#ifndef _H_IRISCONDITIONVARIABLE_
#define _H_IRISCONDITIONVARIABLE_

#include "IrisDevHeader.h"

#include "IrisMutex.h"
#include "IrisConditionVariableTag.h"
#include "IrisInterpreter/IrisStructure/IrisObject.h"

class IrisConditionVariable : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {

		if (!IrisDevUtil::CheckClass((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0), "Mutex")) {
			IrisDevUtil::GroanIrregularWithString("Invalid Parameter.");
			return IrisDevUtil::Nil();
		}

		auto pMutex = &IrisDevUtil::GetNativePointer<IrisMutexTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0))->GetMutex();
		auto pConditionVariable = IrisDevUtil::GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->Initialize(pMutex);
		static_cast<IrisObject*>(ivObj.GetIrisObject())->AddInstanceValue("@mutex", (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
		return ivObj;
	}

	static IrisValue NotifyAll(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDevUtil::GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->NotifyAll();
		return IrisDevUtil::Nil();
	}

	static IrisValue NotifyOne(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDevUtil::GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->NotifyOne();
		return IrisDevUtil::Nil();
	}

	static IrisValue Wait(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pConditionVariable = IrisDevUtil::GetNativePointer<IrisConditionVariableTag*>(ivObj);
		pConditionVariable->Wait();
		return IrisDevUtil::Nil();
	}

public:
	
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "ConditionVariable";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "notify_one", NotifyOne, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "notify_all", NotifyAll, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "wait", Wait, 0, false);
	}

	IrisConditionVariable();
	~IrisConditionVariable();
};

#endif