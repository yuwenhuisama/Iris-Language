#include "IrisInterpreter/IrisNativeClasses/IrisHashIterator.h"



IrisValue IrisHashIterator::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisHashTag* pHash = IrisDevUtil::GetNativePointer<IrisHashTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->Initialize(&pHash->m_mpHash);
	return ivObj;
}

IrisValue IrisHashIterator::Next(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisHashIterator::IsEnd(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->IsEnd() ? IrisInterpreter::CurrentInterpreter()->True() : IrisInterpreter::CurrentInterpreter()->False();
}

IrisValue IrisHashIterator::GetKey(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->GetKey();
}

IrisValue IrisHashIterator::GetValue(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->GetValue();
}

IrisHashIterator::IrisHashIterator()
{
}


IrisHashIterator::~IrisHashIterator()
{
}
