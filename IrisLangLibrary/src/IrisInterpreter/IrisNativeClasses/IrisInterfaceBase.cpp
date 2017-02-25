#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"


IrisValue IrisInterfaceBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisInterfaceBase::GetInterfaceName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisInterfaceBaseTag* pInterface = IrisDevUtil::GetNativePointer<IrisInterfaceBaseTag*>(ivObj);
	const string& strModuleName = pInterface->GetInterfaceName();
	return IrisDevUtil::CreateString(strModuleName.c_str());
}

IrisInterfaceBase::IrisInterfaceBase()
{
}


IrisInterfaceBase::~IrisInterfaceBase()
{
}
