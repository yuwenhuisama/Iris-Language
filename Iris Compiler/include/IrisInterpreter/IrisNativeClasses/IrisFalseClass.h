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
		IrisFalseClassTag* pTrue = IrisDev_GetNativePointer<IrisFalseClassTag*>(ivObj);
		return IrisDev_CreateString(pTrue->GetName().c_str());
	}

	static IrisValue LogicNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_True();
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
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "name", GetName, 0, false);
		IrisDev_AddInstanceMethod(this, "!", LogicNot, 0, false);
	}

	const char* NativeClassNameDefine() const {
		return "FalseClass";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	IrisFalseClass();
	~IrisFalseClass();
};

#endif