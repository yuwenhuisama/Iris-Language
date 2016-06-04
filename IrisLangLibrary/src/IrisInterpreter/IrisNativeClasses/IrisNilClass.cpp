#include "IrisInterpreter/IrisNativeClasses/IrisNilClass.h"


IrisValue IrisNilClass::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisNilClass::GetName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisNilClassTag* pNil = IrisDevUtil::GetNativePointer<IrisNilClassTag*>(ivObj);
	return IrisDevUtil::CreateString(pNil->GetName().c_str());
}

IrisValue IrisNilClass::LogicNot(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::True();
}

IrisNilClass::IrisNilClass()
{
}


IrisNilClass::~IrisNilClass()
{
}
