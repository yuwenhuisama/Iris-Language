#include "IrisInterpreter/IrisNativeClasses/IrisHashIterator.h"



IrisValue IrisHashIterator::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisHashTag* pHash = IrisDevUtil::GetNativePointer<IrisHashTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->Initialize(&pHash->m_mpHash);
	return ivObj;
}

IrisValue IrisHashIterator::Next(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisHashIterator::IsEnd(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->IsEnd() ? IrisInterpreter::CurrentInterpreter()->True() : IrisInterpreter::CurrentInterpreter()->False();
}

IrisValue IrisHashIterator::GetKey(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->GetKey();
}

IrisValue IrisHashIterator::GetValue(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisHashIteratorTag*>(ivObj)->GetValue();
}

IrisHashIterator::IrisHashIterator()
{
}


IrisHashIterator::~IrisHashIterator()
{
}
