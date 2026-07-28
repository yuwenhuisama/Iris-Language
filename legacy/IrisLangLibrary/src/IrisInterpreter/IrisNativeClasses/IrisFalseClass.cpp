#include "IrisInterpreter/IrisNativeClasses/IrisFalseClass.h"


IrisValue IrisFalseClass::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisFalseClass::GetName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisFalseClassTag* pTrue = IrisDevUtil::GetNativePointer<IrisFalseClassTag*>(ivObj);
	return IrisDevUtil::CreateString(pTrue->GetName().c_str());
}

IrisValue IrisFalseClass::LogicNot(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::True();
}

IrisFalseClass::IrisFalseClass()
{
}


IrisFalseClass::~IrisFalseClass()
{
}
