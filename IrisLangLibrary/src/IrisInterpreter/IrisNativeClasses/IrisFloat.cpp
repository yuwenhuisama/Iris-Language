#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"


IrisValue IrisFloat::CmpOperation(Operation eOperationType, IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisFloatTag iftRightFloat;
	// 如果右边为Integer，则转化为Float之间的运算
	if (IrisDevUtil::CheckClassIsInteger((IrisValue&)ivsValues->GetValue(0))) {
		// 获取右边的值
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
		// 将右边对象转换为Float
		iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
	}
	else {
		if (IrisDevUtil::CheckClassIsFloat((IrisValue&)ivsValues->GetValue(0))) {
			iftRightFloat = *(IrisDevUtil::GetNativePointer<IrisFloatTag*>((IrisValue&)ivsValues->GetValue(0)));
		}
		else {
			IrisDevUtil::GroanIrregularWithString("Invaid parameter was sent.");
			return IrisDevUtil::Nil();
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

IrisValue IrisFloat::CastOperation(Operation eOperationType, IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisFloatTag iftRightFloat;
	// 如果右边为Integer，则转化为Float之间的运算
	if (IrisDevUtil::CheckClassIsInteger((IrisValue&)ivsValues->GetValue(0))) {
		// 获取右边的值
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
		// 将右边对象转换为Float
		iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
	}
	else {
		iftRightFloat = *(IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivsValues->GetValue(0)));
	}
	// 新建临时Float对象作为结果
	//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Float"), nullptr, pContextEnvironment);
	ivValue = IrisDevUtil::CreateFloat(0.0f);
	// 获取新建对象的Native Pointer
	IrisFloatTag* pResultFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivValue);
	// 执行运算并保存数据
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

IrisValue IrisFloat::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisFloat::Add(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Add, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Sub(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Sub, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Mul(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Mul, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Div(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Div, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Power(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Power, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Equal(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::Equal, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::NotEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::NotEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::BigThan(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::BigThanOrEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::LessThan(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::LessThanOrEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisFloat::Plus(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateFloat(pFloat->m_dFloat);
	return ivValue;
}

IrisValue IrisFloat::Minus(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateFloat(-pFloat->m_dFloat);
	return ivValue;
}

IrisValue IrisFloat::ToString(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
	return IrisDevUtil::CreateString(pFloat->ToString().c_str());
}

IrisFloat::IrisFloat()
{
}


IrisFloat::~IrisFloat()
{
}
