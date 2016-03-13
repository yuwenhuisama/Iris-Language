#ifndef _H_IRISTRUECLASS_
#define _H_IRISTRUECLASS_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisTrueClassTag.h"

class IrisTrueClass : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisTrueClassTag* pTrue =  IrisDev_GetNativePointer<IrisTrueClassTag*>(ivObj);
		return IrisDev_CreateString(pTrue->GetName().c_str());
	}

	static IrisValue LogicNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_False();
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "TrueClass";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisTrueClassTag);
	}

	void* NativeAlloc() {
		return new IrisTrueClassTag();
	}

	void NativeFree(void* pNativePointer) {
		delete (IrisTrueClassTag*)pNativePointer;
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "name", GetName, 0, false);
		IrisDev_AddInstanceMethod(this, "!", LogicNot, 0, false);
	}

	IrisTrueClass();
	~IrisTrueClass();
};

#endif