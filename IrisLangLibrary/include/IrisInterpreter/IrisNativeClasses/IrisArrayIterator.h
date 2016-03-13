#ifndef _H_IRISARRAYITERATOR_
#define _H_IRISARRAYITERATOR_

#include "IrisDevHeader.h"
#include "IrisArrayTag.h"
#include "IrisArrayIteratorTag.h"

class IrisArrayIterator : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisArrayTag* pArray = static_cast<IrisArrayTag*>(ivsValues->GetValue(0).GetInstanceNativePointer());
		IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Initialize(&pArray->m_vcValues);
		return ivObj;
	}

	static IrisValue Next(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Next();
		return ivObj;
	}

	static IrisValue IsEnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->IsEnd() ? IrisDevUtil::True() : IrisDevUtil::False();
	}

	static IrisValue GetValue(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->GetValue();
	}

public:
	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisArrayIteratorTag);
	}

	void Mark(void* pNativeObjectPointer) { }

	void* NativeAlloc() {
		return new IrisArrayIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisArrayIteratorTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "next", Next, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}
	
	const char* NativeClassNameDefine() const {
		return "ArrayIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	IrisArrayIterator();
	~IrisArrayIterator();
};

#endif