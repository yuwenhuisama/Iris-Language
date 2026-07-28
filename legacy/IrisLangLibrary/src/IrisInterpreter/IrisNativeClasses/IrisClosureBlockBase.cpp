#include "IrisInterpreter/IrisNativeClasses/IrisClosureBlockBase.h"



IrisValue IrisClosureBlockBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pClosureBlock = IrisDevUtil::GetClosureBlock(pContextEnvironment);
	IrisDevUtil::GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->SetClosureBlock(pClosureBlock);

	static_cast<IrisClosureBlock*>(pClosureBlock)->GetNativeObject()->SetNativeObject(nullptr);
	static_cast<IrisClosureBlock*>(pClosureBlock)->SetNativeObject(static_cast<IrisObject*>(ivObj.GetIrisObject()));

	IrisDevUtil::ContextEnvironmentSetClosureBlock(pContextEnvironment, nullptr);
	return ivObj;
}

IrisValue IrisClosureBlockBase::Call(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pClosureBlock = IrisDevUtil::GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->GetClosureBlock();
	return IrisDevUtil::ExcuteClosureBlock(pClosureBlock, ivsVariableValues, pThreadInfo);
}

IrisClosureBlockBase::IrisClosureBlockBase()
{
}


IrisClosureBlockBase::~IrisClosureBlockBase()
{
}
