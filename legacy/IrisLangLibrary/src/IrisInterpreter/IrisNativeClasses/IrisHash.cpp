#include "IrisInterpreter/IrisNativeClasses/IrisHash.h"



IrisValue IrisHash::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
	return ivObj;
}

IrisValue IrisHash::At(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	const IrisValue& ivIndex = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	return IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->At(ivIndex);
}

IrisValue IrisHash::Set(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	const IrisValue& ivIndex = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	const IrisValue& ivValue = static_cast<IrisValues*>(ivsValues)->GetValue(1);
	return IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Set(ivIndex, ivValue);
}

IrisValue IrisHash::Each(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pClosureBlock = static_cast<IrisClosureBlock*>(static_cast<IrisContextEnvironment*>(pContextEnvironment)->GetClosureBlock());
	IrisValues ivValues(2);
	for (auto& elem : IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->m_mpHash) {
		ivValues[0] = elem.first;
		ivValues[1] = elem.second;
		pClosureBlock->Excute(&ivValues, pThreadInfo);
		if (IrisDevUtil::IrregularHappened(pThreadInfo) || IrisDevUtil::FatalErrorHappened(pThreadInfo)) {
			break;
		}
	}
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisHash::Size(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::CreateInt(IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Size());
}

IrisValue IrisHash::GetIterator(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValues ivsParameter = { ivObj };
	return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("HashIterator"), &ivsParameter, pContextEnvironment, pThreadInfo);
}

IrisHash::IrisHash()
{
}


IrisHash::~IrisHash()
{
}
