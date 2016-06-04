#ifndef _H_IRISTRUECLASS_
#define _H_IRISTRUECLASS_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisTrueClassTag.h"

class IrisTrueClass : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LogicNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "TrueClass";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "name", GetName, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "!", LogicNot, 0, false);
	}

	IrisTrueClass();
	~IrisTrueClass();
};

#endif