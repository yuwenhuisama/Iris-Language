#include "IrisInterpreter/IrisNativeClasses/IrisArray.h"
#include "../../../../Iris Library Test/include/IrisLangLibrary.h"


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

IrisValue IrisArray::Empty(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Empty() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::IndexOf(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(ivsValues->GetValue(0));
	int index = object->IndexOf(ref);
	return IrisDevUtil::CreateInt(index);
}

IrisValue IrisArray::Include(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(ivsValues->GetValue(0));
	return IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj)->Include(ref) ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisValue IrisArray::Merge(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
	auto object = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ivObj);
	IrisValue& ref = const_cast<IrisValue&>(ivsValues->GetValue(0));
	auto another = IrisDevUtil::GetNativePointer<IrisArrayTag*>(ref);
	if (ref == IrisDev_Nil()) return IrisDev_Nil();
	object->Merge(*another);
	return ivObj;
}

IrisArray::IrisArray()
{
}


IrisArray::~IrisArray()
{
}
