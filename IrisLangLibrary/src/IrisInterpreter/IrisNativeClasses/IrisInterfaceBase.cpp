#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"


IrisValue IrisInterfaceBase::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisInterfaceBase::GetInterfaceName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
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
