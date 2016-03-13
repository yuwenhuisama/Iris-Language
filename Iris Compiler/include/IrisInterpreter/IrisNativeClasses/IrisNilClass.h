#ifndef _H_IRISNILCLASS_
#define _H_IRISNILCLASS_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisNilClassTag.h"

class IrisNilClass : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisNilClassTag* pNil = IrisDev_GetNativePointer<IrisNilClassTag*>(ivObj);
		return IrisDev_CreateString(pNil->GetName().c_str());
	}

	static IrisValue LogicNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_True();
	}

public:
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "NilClass";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisNilClassTag);
	}

	void* NativeAlloc() {
		return new IrisNilClassTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisNilClassTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "name", GetName, 0, false);
		IrisDev_AddInstanceMethod(this, "!", LogicNot, 0, false);
	}

	IrisNilClass();
	~IrisNilClass();
};

#endif