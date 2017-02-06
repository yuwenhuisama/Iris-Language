#include "IrisInterpreter/IrisNativeClasses/IrisString.h"


IrisValue IrisString::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisString::Add(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	
	if (!IrisDevUtil::CheckClassIsString(ivValue)) {
		IrisDevUtil::GroanIrregularWithString("String CAN ONLY be added with a String object.");
		return IrisDevUtil::Nil();
	}

	IrisStringTag* pString = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivObj);
	IrisStringTag* pAddedString = IrisDevUtil::GetNativePointer<IrisStringTag*>(const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0)));

	ivValue = IrisDevUtil::CreateString("");
	IrisStringTag* pResultString = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivValue);
	(*pResultString) = pString->Add(*pAddedString);
	return ivValue;
}

IrisValue IrisString::Equal(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pSelf = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivObj);
	auto& ivTarget = (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0);
	if (!IrisDevUtil::CheckClassIsClass(ivTarget)) {
		return IrisDevUtil::False();
	}
	auto pTarget = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivTarget);
	return pSelf->GetString() == pTarget->GetString() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisString::IrisString()
{
}


IrisString::~IrisString()
{
}
