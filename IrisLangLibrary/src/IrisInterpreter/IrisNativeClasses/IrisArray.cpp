#include "IrisInterpreter/IrisNativeClasses/IrisArray.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include <algorithm>  

IrisValue IrisArray::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
	return ivObj;
}

IrisValue IrisArray::At(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto ivIndex = static_cast<IrisValues*>(ivsValues)->GetValue(0); //(*ivsValues)[0];
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.", pThreadInfo);
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->At(IrisInteger::GetIntData(ivIndex));
}

IrisValue IrisArray::Set(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValue& ivIndex = (IrisValue&)(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.", pThreadInfo);
		return IrisDevUtil::Nil();
	}
	IrisValue& ivValue = (IrisValue&)(static_cast<IrisValues*>(ivsValues)->GetValue(1));
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Set(IrisInteger::GetIntData(ivIndex), ivValue);
}

IrisValue IrisArray::Each(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pClosureBlock = IrisDevUtil::GetClosureBlock(pContextEnvironment);

	if (!pClosureBlock) {
		IrisDevUtil::GroanIrregularWithString("The method of each of ARRAY needs a block.", pThreadInfo);
		return IrisDevUtil::Nil();
	}

	IrisValues ivValues(1);
	for (auto& elem : IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->m_vcValues) {
		ivValues[0] = elem;
		IrisDevUtil::ExcuteClosureBlock(pClosureBlock, &ivValues, pThreadInfo);
		if (IrisDevUtil::IrregularHappened(pThreadInfo) || IrisDevUtil::FatalErrorHappened(pThreadInfo)) {
			break;
		}
	}
	return IrisDevUtil::Nil();
}

IrisValue IrisArray::Push(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	if (IrisDevUtil::ObjectIsFixed(ivObj)) {
		IrisDevUtil::GroanIrregularWithString("Cannot push value into a fixed ARRAY object.", pThreadInfo);
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Push((static_cast<IrisValues*>(ivsValues)->GetValue(0)));
}

IrisValue IrisArray::Pop(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	if (IrisDevUtil::ObjectIsFixed(ivObj)) {
		IrisDevUtil::GroanIrregularWithString("Cannot pop value from a fixed ARRAY object.", pThreadInfo);
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Pop();
}

IrisValue IrisArray::Size(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::CreateInt(IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Size());
}

IrisValue IrisArray::GetIterator(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValues ivsParameter = { ivObj };
	return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("ArrayIterator"), &ivsParameter, pContextEnvironment, pThreadInfo);
}

IrisValue IrisArray::Sort(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo)
{
	auto pContext = static_cast<IrisContextEnvironment*>(pContextEnvironment);
	auto pBlock = static_cast<IrisClosureBlock*>(pContext->GetClosureBlock());

	auto pArray = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	size_t nSize = pArray->Size();

	IrisValues ivBlockParam;

	if (nSize <= 1) {
		return IrisDevUtil::Nil();
	}

	auto& vcValues = pArray->m_vcValues;

	//for (auto& value : vcValues) {
	//	bool dummy = false;
	//	if (!IrisDevUtil::GetClassOfObject(value)->GetInternClass()->GetMethod(">", dummy)) {
	//		IrisDevUtil::GroanIrregularWithString("Element of sorted array has not been defined '>' in.", pThreadInfo);
	//		return IrisDevUtil::Nil();
	//	}
	//}

	if (!pBlock) {
		try {
			std::sort(
				vcValues.begin(),
				vcValues.end(),
				[&](IrisValue& ivA, IrisValue& ivB)-> bool {
				ivBlockParam.SetValue(0, ivB);

				auto bResult = IrisDevUtil::CallMethod(ivA, ">", &ivBlockParam, pContext, pThreadInfo);

				if (IrisDevUtil::IrregularHappened(pThreadInfo)
					|| IrisDevUtil::FatalErrorHappened(pThreadInfo)) {
					throw("Error when sorting.");
				}

				return bResult == IrisDevUtil::True();
			}
			);
		}
		catch (string e) {
			return IrisDevUtil::Nil();
		}
	}
	else {
		try {
			std::sort(
				vcValues.begin(),
				vcValues.end(),
				[&](IrisValue& ivA, IrisValue& ivB)-> bool {
				ivBlockParam.SetValue(0, ivA);
				ivBlockParam.SetValue(1, ivB);

				if (IrisDevUtil::IrregularHappened(pThreadInfo)
					|| IrisDevUtil::FatalErrorHappened(pThreadInfo)) {
					throw("Error when sorting.");
				}

				return pBlock->Excute(&ivBlockParam, pThreadInfo) == IrisDevUtil::True();
			}
			);
		}
		catch(string e) {
			return IrisDevUtil::Nil();
		}
	}
	return IrisDevUtil::Nil();
}

IrisValue IrisArray::Empty(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Empty() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::IndexOf(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	int index = object->IndexOf(ref);
	return IrisDevUtil::CreateInt(index);
}

IrisValue IrisArray::Include(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Include(ref) ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::Merge(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	auto another = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ref);
	if (ref == IrisDevUtil::Nil()) return IrisDevUtil::Nil();
	object->Merge(*another);
	return ivObj;
}

IrisArray::IrisArray()
{
}


IrisArray::~IrisArray()
{
}
