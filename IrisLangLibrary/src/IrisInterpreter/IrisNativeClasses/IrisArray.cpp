#include "IrisInterpreter/IrisNativeClasses/IrisArray.h"


IrisValue IrisArray::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
	return ivObj;
}

IrisValue IrisArray::At(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto ivIndex = ivsValues->GetValue(0); //(*ivsValues)[0];
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
		return IrisDevUtil::Nil();
	}
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->At(IrisInteger::GetIntData(ivIndex));
}

IrisValue IrisArray::Set(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue& ivIndex = (IrisValue&)(ivsValues->GetValue(0));
	if (!IrisDevUtil::CheckClassIsInteger(ivIndex)) {
		IrisDevUtil::GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
		return IrisDevUtil::Nil();
	}
	IrisValue& ivValue = (IrisValue&)(ivsValues->GetValue(1));
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
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Push((ivsValues->GetValue(0)));
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

IrisArray::IrisArray()
{
}


IrisArray::~IrisArray()
{
}
