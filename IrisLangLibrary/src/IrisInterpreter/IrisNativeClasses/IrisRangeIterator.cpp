#include "IrisInterpreter/IrisNativeClasses/IrisRangeIterator.h"



IrisValue IrisRangeIterator::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisRangeTag* pRange = IrisDevUtil::GetNativePointer<IrisRangeTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisRangeIteratorTag* pRangeIter = IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj);
	pRangeIter->Initialize(pRange);
	return ivObj;
}

IrisValue IrisRangeIterator::Next(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisRangeIterator::IsEnd(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->IsEnd()
		? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisRangeIterator::GetValue(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->GetValue();
}

IrisRangeIterator::IrisRangeIterator()
{
}

IrisRangeIterator::~IrisRangeIterator()
{
}
