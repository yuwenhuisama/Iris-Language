#include "IrisInterpreter/IrisNativeClasses/IrisTrueClass.h"


IrisValue IrisTrueClass::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisTrueClass::GetName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisTrueClassTag* pTrue = IrisDevUtil::GetNativePointer<IrisTrueClassTag*>(ivObj);
	return IrisDevUtil::CreateString(pTrue->GetName().c_str());
}

IrisValue IrisTrueClass::LogicNot(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::False();
}

IrisTrueClass::IrisTrueClass()
{
}


IrisTrueClass::~IrisTrueClass()
{
}
