#ifndef _H_IRISHASHITERATOR_
#define _H_IRISHASHITERATOR_

#include "IrisDevHeader.h"

#include "IrisHashTag.h"
#include "IrisInteger.h"
#include "IrisHashIteratorTag.h"
#include "IrisInterpreter.h"

class IrisHashIterator :
	public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Next(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue IsEnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetKey(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetValue(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}
	const char* NativeClassNameDefine() const {
		return "HashIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisHashIteratorTag);
	}

	void* NativeAlloc() {
		return new IrisHashIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete (IrisHashIteratorTag*)pNativePointer;
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "next", Next, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_key", GetKey, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}

	IrisHashIterator();
	~IrisHashIterator();
};

#endif