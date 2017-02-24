#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"


IrisValue IrisFloat::CmpOperation(Operation eOperationType, const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisFloatTag iftRightFloat;
	// ����ұ�ΪInteger����ת��ΪFloat֮�������
	if (IrisDevUtil::CheckClassIsInteger((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
		// ��ȡ�ұߵ�ֵ
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// ���ұ߶���ת��ΪFloat
		iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
	}
	else {
		if (IrisDevUtil::CheckClassIsFloat((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
			iftRightFloat = *(IrisDevUtil::GetNativePointer<IrisFloatTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0)));
		}
		else {
			//IrisDevUtil::GroanIrregularWithString("Invaid parameter was sent.");
			return IrisDevUtil::False();
		}
	}
	bool bResult = false;
	switch (eOperationType)
	{
	case IrisFloat::Operation::Equal:
		bResult = pFloat->Equal(iftRightFloat);
		break;
	case IrisFloat::Operation::NotEqual:
		bResult = pFloat->NotEqual(iftRightFloat);
		break;
	case IrisFloat::Operation::BigThan:
		bResult = pFloat->BigThan(iftRightFloat);
		break;
	case IrisFloat::Operation::BigThanOrEqual:
		bResult = pFloat->BigThanOrEqual(iftRightFloat);
		break;
	case IrisFloat::Operation::LessThan:
		bResult = pFloat->LessThan(iftRightFloat);
		break;
	case IrisFloat::Operation::LessThanOrEqual:
		bResult = pFloat->LessThanOrEqual(iftRightFloat);
		break;
	default:
		break;
	}
	if (bResult) {
		return IrisInterpreter::CurrentInterpreter()->True();
	}
	else {
		return IrisInterpreter::CurrentInterpreter()->False();
	}
}

IrisValue IrisFloat::CastOperation(Operation eOperationType, const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisFloatTag iftRightFloat;
	// ����ұ�ΪInteger����ת��ΪFloat֮�������
	if (IrisDevUtil::CheckClassIsInteger((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
		// ��ȡ�ұߵ�ֵ
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// ���ұ߶���ת��ΪFloat
		iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
	}
	else {
		iftRightFloat = *(IrisDevUtil::GetNativePointer<IrisFloatTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0)));
	}
	// �½���ʱFloat������Ϊ���
	//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Float"), nullptr, pContextEnvironment);
	ivValue = IrisDevUtil::CreateFloat(0.0f);
	// ��ȡ�½������Native Pointer
	IrisFloatTag* pResultFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivValue);
	// ִ�����㲢��������
	switch (eOperationType)
	{
	case(Operation::Add) :
		(*pResultFloat) = pFloat->Add(iftRightFloat);
		break;
	case(Operation::Sub) :
		(*pResultFloat) = pFloat->Sub(iftRightFloat);
		break;
	case(Operation::Mul) :
		(*pResultFloat) = pFloat->Mul(iftRightFloat);
		break;
	case(Operation::Div) :
		(*pResultFloat) = pFloat->Div(iftRightFloat);
		break;
	case(Operation::Power) :
		(*pResultFloat) = pFloat->Power(iftRightFloat);
		break;
	default:
		break;
	}
	return ivValue;
}

IrisValue IrisFloat::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisFloat::Add(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Add, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Sub(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Sub, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Mul(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Mul, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Div(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Div, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Power(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Power, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Equal(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::Equal, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::NotEqual(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::NotEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::BigThan(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::BigThanOrEqual(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::LessThan(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::LessThanOrEqual(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Plus(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateFloat(pFloat->m_dFloat);
	return ivValue;
}

IrisValue IrisFloat::Minus(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateFloat(-pFloat->m_dFloat);
	return ivValue;
}

IrisValue IrisFloat::ToString(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	return IrisDevUtil::CreateString(pFloat->ToString().c_str());
}

IrisValue IrisFloat::ToInteger(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto fFloat = IrisDevUtil::GetFloat(ivObj);
	return IrisDevUtil::CreateFloat(static_cast<int>(fFloat));
}

IrisFloat::IrisFloat()
{
}


IrisFloat::~IrisFloat()
{
}
