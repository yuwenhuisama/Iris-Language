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
		IrisDev_GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Initialize(&pArray->m_vcValues);
		return ivObj;
	}

	static IrisValue Next(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDev_GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Next();
		return ivObj;
	}

	static IrisValue IsEnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_GetNativePointer<IrisArrayIteratorTag*>(ivObj)->IsEnd() ? IrisDev_True() : IrisDev_False();
	}

	static IrisValue GetValue(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_GetNativePointer<IrisArrayIteratorTag*>(ivObj)->GetValue();
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
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDev_AddInstanceMethod(this, "next", Next, 0, false);
		IrisDev_AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDev_AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}
	
	const char* NativeClassNameDefine() const {
		return "ArrayIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	IrisArrayIterator();
	~IrisArrayIterator();
};

#endif