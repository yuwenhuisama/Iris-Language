#include "IrisInterpreter/IrisNativeClasses/IrisRange.h"



IrisValue IrisRange::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto* pRange = IrisDevUtil::GetNativePointer<IrisRangeTag*>(ivObj);
	auto ivValue = static_cast<IrisValues*>(ivsValues)->GetValue(0);

	if (!IrisDevUtil::CheckClassIsInteger(ivValue)) {
		IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
		return IrisDevUtil::Nil();
	}

	auto nType = IrisDevUtil::GetInt(ivValue);
	if (nType < 0 || nType > 3) {
		IrisDevUtil::GroanIrregularWithString("Invaid parameter of method __format of class Range.");
		return IrisDevUtil::Nil();
	}

	auto eType = static_cast<IrisRangeType>(IrisDevUtil::GetInt(ivValue));

	ivValue = static_cast<IrisValues*>(ivsValues)->GetValue(1);
	auto ivValue2 = static_cast<IrisValues*>(ivsValues)->GetValue(2);
	if (IrisDevUtil::CheckClassIsStringOrUniqueString(const_cast<IrisValue&>(ivValue)) && IrisDevUtil::CheckClassIsStringOrUniqueString(ivValue2)) {
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
	else if (IrisDevUtil::CheckClassIsInteger(ivValue) && IrisDevUtil::CheckClassIsInteger(ivValue2)) {
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

IrisValue IrisRange::GetIterator(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValues ivsParameter = { ivObj };
	return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("RangeIterator"), &ivsParameter, pContextEnvironment);
}

IrisRange::IrisRange()
{
}

IrisRange::~IrisRange()
{
}
