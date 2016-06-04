#include "IrisInterpreter/IrisNativeClasses/IrisTrueClass.h"


IrisValue IrisTrueClass::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisTrueClass::GetName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisTrueClassTag* pTrue = IrisDevUtil::GetNativePointer<IrisTrueClassTag*>(ivObj);
	return IrisDevUtil::CreateString(pTrue->GetName().c_str());
}

IrisValue IrisTrueClass::LogicNot(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::False();
}

IrisTrueClass::IrisTrueClass()
{
}


IrisTrueClass::~IrisTrueClass()
{
}
