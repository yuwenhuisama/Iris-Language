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
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisHashTag* pHash = (IrisHashTag*)(ivsValues->GetValue(0).GetInstanceNativePointer());
		((IrisHashIteratorTag*)ivObj.GetInstanceNativePointer())->Initialize(&pHash->m_mpHash);
		return ivObj;
	}

	static IrisValue Next(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		((IrisHashIteratorTag*)ivObj.GetInstanceNativePointer())->Next();
		return ivObj;
	}

	static IrisValue IsEnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ((IrisHashIteratorTag*)ivObj.GetInstanceNativePointer())->IsEnd() ? IrisInterpreter::CurrentInterpreter()->True() : IrisInterpreter::CurrentInterpreter()->False();
	}

	static IrisValue GetKey(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ((IrisHashIteratorTag*)ivObj.GetInstanceNativePointer())->GetKey();
	}

	static IrisValue GetValue(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ((IrisHashIteratorTag*)ivObj.GetInstanceNativePointer())->GetValue();
	}

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