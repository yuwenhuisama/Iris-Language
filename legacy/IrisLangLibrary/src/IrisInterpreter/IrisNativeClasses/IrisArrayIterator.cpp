#include "IrisInterpreter/IrisNativeClasses/IrisArrayIterator.h"



IrisValue IrisArrayIterator::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisArrayTag* pArray = IrisDevUtil::GetNativePointer<IrisArrayTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Initialize(&pArray->m_vcValues);
	return ivObj;
}

IrisValue IrisArrayIterator::Next(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisArrayIterator::IsEnd(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->IsEnd() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArrayIterator::GetValue(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->GetValue();
}

IrisArrayIterator::IrisArrayIterator()
{
}


IrisArrayIterator::~IrisArrayIterator()
{
}
