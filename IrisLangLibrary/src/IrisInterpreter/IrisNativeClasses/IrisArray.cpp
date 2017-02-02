#include "IrisInterpreter/IrisNativeClasses/IrisArray.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include <algorithm>  

IrisValue IrisArray::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
	return ivObj;
}

IrisValue IrisArray::At(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto ivIndex = static_cast<IrisValues*>(ivsValues)->GetValue(0); //(*ivsValues)[0];
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->At(IrisInteger::GetIntData(ivIndex));
}

IrisValue IrisArray::Set(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue& ivIndex = (IrisValue&)(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
		return IrisDevUtil::Nil();
	}
	IrisValue& ivValue = (IrisValue&)(static_cast<IrisValues*>(ivsValues)->GetValue(1));
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Set(IrisInteger::GetIntData(ivIndex), ivValue);
}

IrisValue IrisArray::Each(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pClosureBlock = IrisDevUtil::GetClosureBlock(pContextEnvironment);

	if (!pClosureBlock) {
		IrisDevUtil::GroanIrregularWithString("The method of each of ARRAY needs a block.");
		return IrisDevUtil::Nil();
	}

	IrisValues ivValues(1);
	for (auto& elem : IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->m_vcValues) {
		ivValues[0] = elem;
		IrisDevUtil::ExcuteClosureBlock(pClosureBlock, &ivValues);
		if (IrisDevUtil::IrregularHappened() || IrisDevUtil::FatalErrorHappened()) {
			break;
		}
	}
	return IrisDevUtil::Nil();
}

IrisValue IrisArray::Push(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	if (IrisDevUtil::ObjectIsFixed(ivObj)) {
		IrisDevUtil::GroanIrregularWithString("Cannot push value into a fixed ARRAY object.");
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Push((static_cast<IrisValues*>(ivsValues)->GetValue(0)));
}

IrisValue IrisArray::Pop(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	if (IrisDevUtil::ObjectIsFixed(ivObj)) {
		IrisDevUtil::GroanIrregularWithString("Cannot pop value from a fixed ARRAY object.");
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Pop();
}

IrisValue IrisArray::Size(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return IrisDevUtil::CreateInt(IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Size());
}

IrisValue IrisArray::GetIterator(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValues ivsParameter = { ivObj };
	return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("ArrayIterator"), &ivsParameter, pContextEnvironment);
}

IrisValue IrisArray::Sort(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
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
	//		IrisDevUtil::GroanIrregularWithString("Element of sorted array has not been defined '>' in.");
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

				auto bResult = IrisDevUtil::CallMethod(ivA, &ivBlockParam, ">");

				if (IrisDevUtil::IrregularHappened()
					|| IrisDevUtil::FatalErrorHappened()) {
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

				if (IrisDevUtil::IrregularHappened()
					|| IrisDevUtil::FatalErrorHappened()) {
					throw("Error when sorting.");
				}

				return pBlock->Excute(&ivBlockParam) == IrisDevUtil::True();
			}
			);
		}
		catch(string e) {
			return IrisDevUtil::Nil();
		}
	}
	return IrisDevUtil::Nil();
}

IrisValue IrisArray::Empty(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Empty() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::IndexOf(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	int index = object->IndexOf(ref);
	return IrisDevUtil::CreateInt(index);
}

IrisValue IrisArray::Include(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Include(ref) ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::Merge(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
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
