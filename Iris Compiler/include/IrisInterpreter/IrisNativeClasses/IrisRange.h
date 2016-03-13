#ifndef _H_IRISRANGE_
#define _H_IRISRANGE_

#include "IrisDevHeader.h"

#include "IrisRangeTag.h"
#include "IrisInteger.h"

class IrisRange : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto* pRange = IrisDev_GetNativePointer<IrisRangeTag*>(ivObj); 
		auto ivValue = ivsValues->GetValue(0);

		if (!IrisDev_CheckClass(ivValue, "Integer")) {
			IrisDev_GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDev_Nil();
		}

		auto nType = IrisDev_GetInt(ivValue);
		if (nType < 0 || nType > 3) {
			IrisDev_GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDev_Nil();
		}

		auto eType = static_cast<IrisRangeType>(IrisDev_GetInt(ivValue));

		ivValue = ivsValues->GetValue(1);
		auto ivValue2 = ivsValues->GetValue(2);
		if (IrisDev_CheckClass((IrisValue&)ivValue, "String") && IrisDev_CheckClass(ivValue2, "String")) {
			auto strFrom = IrisDev_GetString(ivValue);
			auto strTo = IrisDev_GetString(ivValue2);
			if (strnlen_s(strFrom, 0xFFFFFFFF) != 1 || strnlen_s(strTo, 0xFFFFFFFF) != 1) {
				IrisDev_GroanIrregularWithString("Invaid parameter of method __format of class Range.");
				return IrisDev_Nil();
			}
			else {
				auto cFrom = strFrom[0];
				auto cTo = strTo[0];
				pRange->Initialize(eType, cFrom, cTo);
			}
		}
		else if (IrisDev_CheckClass(ivValue, "Integer") && IrisDev_CheckClass(ivValue2, "Integer")) {
			auto nFrom = IrisDev_GetInt(ivValue);
			auto nTo = IrisDev_GetInt(ivValue2);
			pRange->Initialize(eType, nFrom, nTo);
		}
		else {
			IrisDev_GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDev_Nil();
		}

		return ivObj;
	}

	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValues ivsParameter = { ivObj };
		return IrisDev_CreateInstance(IrisDev_GetClass("RangeIterator"), &ivsParameter, pContextEnvironment);
	}

public:
	
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Range";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisRangeTag);
	}

	void* NativeAlloc() {
		return new IrisRangeTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisRangeTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 3, false);
		IrisDev_AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);
	}
	IrisRange();
	~IrisRange();
};

#endif