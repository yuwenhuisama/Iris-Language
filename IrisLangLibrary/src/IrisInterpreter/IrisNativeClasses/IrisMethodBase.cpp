#include "IrisInterpreter/IrisNativeClasses/IrisMethodBase.h"


IrisValue IrisMethodBase::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisMethodBase::GetMethodName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
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
