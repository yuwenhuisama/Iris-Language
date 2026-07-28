#include "IrisInterpreter/IrisNativeClasses/IrisRangeIterator.h"



IrisValue IrisRangeIterator::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisRangeTag* pRange = IrisDevUtil::GetNativePointer<IrisRangeTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	IrisRangeIteratorTag* pRangeIter = IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj);
	pRangeIter->Initialize(pRange);
	return ivObj;
}

IrisValue IrisRangeIterator::Next(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->Next();
	return ivObj;
}

IrisValue IrisRangeIterator::IsEnd(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->IsEnd()
		? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisRangeIterator::GetValue(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::GetNativePointer<IrisRangeIteratorTag*>(ivObj)->GetValue();
}

IrisRangeIterator::IrisRangeIterator()
{
}

IrisRangeIterator::~IrisRangeIterator()
{
}
