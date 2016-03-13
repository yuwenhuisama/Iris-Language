#ifndef _H_IRISRANGE_
#define _H_IRISRANGE_

#include "IrisDevHeader.h"

#include "IrisRangeTag.h"
#include "IrisInteger.h"

class IrisRange : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto* pRange = IrisDevUtil::GetNativePointer<IrisRangeTag*>(ivObj); 
		auto ivValue = ivsValues->GetValue(0);

		if (!IrisDevUtil::CheckClass(ivValue, "Integer")) {
			IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDevUtil::Nil();
		}

		auto nType = IrisDevUtil::GetInt(ivValue);
		if (nType < 0 || nType > 3) {
			IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDevUtil::Nil();
		}

		auto eType = static_cast<IrisRangeType>(IrisDevUtil::GetInt(ivValue));

		ivValue = ivsValues->GetValue(1);
		auto ivValue2 = ivsValues->GetValue(2);
		if (IrisDevUtil::CheckClass((IrisValue&)ivValue, "String") && IrisDevUtil::CheckClass(ivValue2, "String")) {
			auto strFrom = IrisDevUtil::GetString(ivValue);
			auto strTo = IrisDevUtil::GetString(ivValue2);
			if (strnlen_s(strFrom, 0xFFFFFFFF) != 1 || strnlen_s(strTo, 0xFFFFFFFF) != 1) {
				IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
				return IrisDevUtil::Nil();
			}
			else {
				auto cFrom = strFrom[0];
				auto cTo = strTo[0];
				pRange->Initialize(eType, cFrom, cTo);
			}
		}
		else if (IrisDevUtil::CheckClass(ivValue, "Integer") && IrisDevUtil::CheckClass(ivValue2, "Integer")) {
			auto nFrom = IrisDevUtil::GetInt(ivValue);
			auto nTo = IrisDevUtil::GetInt(ivValue2);
			pRange->Initialize(eType, nFrom, nTo);
		}
		else {
			IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
			return IrisDevUtil::Nil();
		}

		return ivObj;
	}

	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValues ivsParameter = { ivObj };
		return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("RangeIterator"), &ivsParameter, pContextEnvironment);
	}

public:
	
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Range";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 3, false);
		IrisDevUtil::AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);
	}
	IrisRange();
	~IrisRange();
};

#endif