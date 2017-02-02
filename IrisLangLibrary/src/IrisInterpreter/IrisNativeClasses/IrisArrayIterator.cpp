#include "IrisInterpreter/IrisNativeClasses/IrisArrayIterator.h"



IrisValue IrisArrayIterator::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisArrayTag* pArray = IrisDevUtil::GetNativePointer<IrisArrayTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Initialize(&pArray->m_vcValues);
	return ivObj;
}

IrisValue IrisArrayIterator::Next(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisArrayIterator::IsEnd(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->IsEnd() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArrayIterator::GetValue(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisArrayIteratorTag*>(ivObj)->GetValue();
}

IrisArrayIterator::IrisArrayIterator()
{
}


IrisArrayIterator::~IrisArrayIterator()
{
}
