#ifndef _H_IRISRANGEITERATOR_
#define _H_IRISRANGEITERATOR_

#include "IrisDevHeader.h"
#include "IrisRangeTag.h"
#include "IrisRangeIteratorTag.h"

class IrisRangeIterator : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisRangeTag* pRange = IrisDev_GetNativePointer<IrisRangeTag*>((IrisValue&)ivsValues->GetValue(0));
		IrisRangeIteratorTag* pRangeIter = IrisDev_GetNativePointer<IrisRangeIteratorTag*>(ivObj);
		pRangeIter->Initialize(pRange);
		return ivObj;
	}

	static IrisValue Next(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDev_GetNativePointer<IrisRangeIteratorTag*>(ivObj)->Next();
		return ivObj;
	}

	static IrisValue IsEnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_GetNativePointer<IrisRangeIteratorTag*>(ivObj)->IsEnd() 
			? IrisDev_True() : IrisDev_False();
	}

	static IrisValue GetValue(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_GetNativePointer<IrisRangeIteratorTag*>(ivObj)->GetValue();
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "RangeIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisRangeIteratorTag);
	}

	void* NativeAlloc() {
		return new IrisRangeIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisRangeIteratorTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDev_AddInstanceMethod(this, "next", Next, 0, false);
		IrisDev_AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDev_AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}

	IrisRangeIterator();
	~IrisRangeIterator();
};

#endif