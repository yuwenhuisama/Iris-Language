#include "IrisInterpreter/IrisNativeClasses/IrisModuleBase.h"


IrisValue IrisModuleBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisModuleBase::GetModuleName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisModuleBaseTag* pModule = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(ivObj);
	const string& strModuleName = pModule->GetModuleName();
	return IrisDevUtil::CreateString(strModuleName.c_str());
}

IrisModuleBase::IrisModuleBase()
{
}


IrisModuleBase::~IrisModuleBase()
{
}
