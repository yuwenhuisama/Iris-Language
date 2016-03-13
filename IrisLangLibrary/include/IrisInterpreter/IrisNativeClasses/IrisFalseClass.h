#ifndef _H_IRISFALSECLASS_
#define _H_IRISFALSECLASS_

#include "IrisDevHeader.h"
#include "IrisFalseClassTag.h"

class IrisFalseClass : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisFalseClassTag* pTrue = IrisDevUtil::GetNativePointer<IrisFalseClassTag*>(ivObj);
		return IrisDevUtil::CreateString(pTrue->GetName().c_str());
	}

	static IrisValue LogicNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDevUtil::True();
	}

public:
	
	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisFalseClassTag);
	}

	void Mark(void* pNativeObjectPointer) {}

	void* NativeAlloc() {
		return new IrisFalseClassTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisFalseClassTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "name", GetName, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "!", LogicNot, 0, false);
	}

	const char* NativeClassNameDefine() const {
		return "FalseClass";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	IrisFalseClass();
	~IrisFalseClass();
};

#endif