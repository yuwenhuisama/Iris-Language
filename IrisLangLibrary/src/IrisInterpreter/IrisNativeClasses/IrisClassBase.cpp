#include "IrisInterpreter/IrisNativeClasses/IrisClassBase.h"


IrisValue IrisClassBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisClassBase::GetClassName(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	// ��ȡ��ָ��
	IrisClassBaseTag* pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivObj);
	const string& strClassName = pClass->GetThisClassName();
	//����String
	return IrisDevUtil::CreateInstanceByInstantValue(strClassName.c_str());
}

IrisValue IrisClassBase::New(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisClassBaseTag* pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivObj);
	return IrisDevUtil::CreateInstance(pClass->GetClass()->GetExternClass(), ivsVariableValues, pContextEnvironment, pThreadInfo);
}

IrisClassBase::IrisClassBase()
{
}


IrisClassBase::~IrisClassBase()
{
}
