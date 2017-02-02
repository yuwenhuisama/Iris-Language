#include "IrisInterpreter/IrisNativeClasses/IrisString.h"


IrisValue IrisString::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisString::Add(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisStringTag* pString = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivObj);
	if (!IrisDevUtil::CheckClassIsString(static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
		IrisDevUtil::GroanIrregularWithString("String CAN ONLY be added with a String object.");
		return IrisDevUtil::Nil();
	}
	IrisStringTag* pAddedString = IrisDevUtil::GetNativePointer<IrisStringTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
	//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("String"), nullptr, pContextEnvironment);
	IrisDevUtil::CreateString("");
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
