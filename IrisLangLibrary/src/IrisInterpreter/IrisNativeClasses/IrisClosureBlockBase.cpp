#include "IrisInterpreter/IrisNativeClasses/IrisClosureBlockBase.h"



IrisValue IrisClosureBlockBase::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pClosureBlock = IrisDevUtil::GetClosureBlock(pContextEnvironment);
	IrisDevUtil::GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->SetClosureBlock(pClosureBlock);
	IrisDevUtil::ContextEnvironmentSetClosureBlock(pContextEnvironment, nullptr);
	return ivObj;
}

IrisValue IrisClosureBlockBase::Call(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pClosureBlock = IrisDevUtil::GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->GetClosureBlock();
	return IrisDevUtil::ExcuteClosureBlock(pClosureBlock, ivsVariableValues);
}

IrisClosureBlockBase::IrisClosureBlockBase()
{
}


IrisClosureBlockBase::~IrisClosureBlockBase()
{
}
