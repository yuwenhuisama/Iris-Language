#include "IrisInterpreter/IrisNativeClasses/IrisMethodBase.h"


IrisValue IrisMethodBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisMethodBase::GetMethodName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisMethodBaseTag* pMethod = IrisDevUtil::GetNativePointer<IrisMethodBaseTag*>(ivObj);
	const string& strMethodName = pMethod->GetMethodName();
	return IrisDevUtil::CreateString(strMethodName.c_str());
}

IrisMethodBase::IrisMethodBase()
{
}


IrisMethodBase::~IrisMethodBase()
{
}
