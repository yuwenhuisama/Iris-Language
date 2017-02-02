#include "IrisInterpreter/IrisNativeClasses/IrisInteger.h"


IrisValue IrisInteger::CmpOperation(Operation eOperationType, IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	bool bResult = false;
	// 如果右边为Float，则转化为Float之间的运算
	if (IrisDevUtil::CheckClassIsFloat(static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
		// 获取被加数
		IrisFloatTag* pRightFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// 将当前对象转换为Float
		IrisFloatTag& iftLeftFloat = static_cast<IrisFloatTag>(*pInteger);
		// 执行运算并保存数据
		switch (eOperationType)
		{
		case(Operation::Equal) :
			bResult = iftLeftFloat.Equal(*pRightFloat);
			break;
		case(Operation::NotEqual) :
			bResult = iftLeftFloat.NotEqual(*pRightFloat);
			break;
		case(Operation::BigThan) :
			bResult = iftLeftFloat.BigThan(*pRightFloat);
			break;
		case(Operation::BigThanOrEqual) :
			bResult = iftLeftFloat.BigThanOrEqual(*pRightFloat);
			break;
		case(Operation::LessThan) :
			bResult = iftLeftFloat.LessThan(*pRightFloat);
			break;
		case(Operation::LessThanOrEqual) :
			bResult = iftLeftFloat.LessThanOrEqual(*pRightFloat);
			break;
		default:
			break;
		}
	}
	else {
		if (!IrisDevUtil::CheckClassIsInteger(static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
			IrisDevUtil::GroanIrregularWithString("Invaid parameter was sent.");
			return IrisDevUtil::Nil();
		}
		// 获取被加数
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// 执行并保存数据
		switch (eOperationType)
		{
		case(Operation::Equal) :
			bResult = pInteger->Equal(*pRightInteger);
			break;
		case(Operation::NotEqual) :
			bResult = pInteger->NotEqual(*pRightInteger);
			break;
		case(Operation::BigThan) :
			bResult = pInteger->BigThan(*pRightInteger);
			break;
		case(Operation::BigThanOrEqual) :
			bResult = pInteger->BigThanOrEqual(*pRightInteger);
			break;
		case(Operation::LessThan) :
			bResult = pInteger->LessThan(*pRightInteger);
			break;
		case(Operation::LessThanOrEqual) :
			bResult = pInteger->LessThanOrEqual(*pRightInteger);
			break;
		default:
			break;
		}
	}
	if (bResult) {
		return IrisDevUtil::True();
	}
	else {
		return IrisDevUtil::False();
	}
}

IrisValue IrisInteger::CastOperation(Operation eOperationType, IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	// 如果右边为Float，则转化为Float之间的运算
	if (IrisDevUtil::CheckClassIsFloat(static_cast<IrisValues*>(ivsValues)->GetValue(0))) {
		// 获取被加数
		IrisFloatTag* pRightFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// 将当前对象转换为Float
		IrisFloatTag& iftLeftFloat = static_cast<IrisFloatTag>(*pInteger);
		if (eOperationType != Operation::Mod) {
			// 新建临时Float对象作为结果
			ivValue = IrisDevUtil::CreateFloat(0.0f);
			// 获取新建对象的Native Pointer
			IrisFloatTag* pResultFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivValue);
			// 执行运算并保存数据
			switch (eOperationType)
			{
			case(Operation::Add) :
				(*pResultFloat) = iftLeftFloat.Add(*pRightFloat);
				break;
			case(Operation::Sub) :
				(*pResultFloat) = iftLeftFloat.Sub(*pRightFloat);
				break;
			case(Operation::Mul) :
				(*pResultFloat) = iftLeftFloat.Mul(*pRightFloat);
				break;
			case(Operation::Div) :
				(*pResultFloat) = iftLeftFloat.Div(*pRightFloat);
				break;
			case(Operation::Power) :
				(*pResultFloat) = iftLeftFloat.Power(*pRightFloat);
				break;
			default:
				break;
			}
		}
		else {
			// 新建临时Integer对象作为结果
			//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
			ivValue = IrisDevUtil::CreateInt(0);
			// 获取新建对象的Native Pointer
			IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
			(*pResultInteger) = pInteger->Mod(static_cast<IrisIntegerTag>(*pRightFloat));
		}
	}
	else {
		// 获取被加数
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
		// 新建临时Integer对象作为结果
		//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
		ivValue = IrisDevUtil::CreateInt(0);
		// 获取新建对象的Native Pointer
		IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
		// 执行并保存数据
		switch (eOperationType)
		{
		case(Operation::Add) :
			(*pResultInteger) = pInteger->Add(*pRightInteger);
			break;
		case(Operation::Sub) :
			(*pResultInteger) = pInteger->Sub(*pRightInteger);
			break;
		case(Operation::Mul) :
			(*pResultInteger) = pInteger->Mul(*pRightInteger);
			break;
		case(Operation::Div) :
			(*pResultInteger) = pInteger->Div(*pRightInteger);
			break;
		case(Operation::Power) :
			(*pResultInteger) = pInteger->Power(*pRightInteger);
			break;
		case(Operation::Mod) :
			(*pResultInteger) = pInteger->Mod(*pRightInteger);
			break;
		default:
			break;
		}
	}
	return ivValue;
}

IrisValue IrisInteger::Operate(Operation eOperationType, IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	// 获取Right
	IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0));
	// 新建临时Integer对象作为结果
	//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
	ivValue = IrisDevUtil::CreateInt(0);
	// 获取新建对象的Native Pointer
	IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
	// 执行并保存数据
	switch (eOperationType)
	{
	case(Operation::Shr) :
		(*pResultInteger) = pInteger->Shr(*pRightInteger);
		break;
	case(Operation::Shl) :
		(*pResultInteger) = pInteger->Shl(*pRightInteger);
		break;
	case(Operation::Sar) :
		(*pResultInteger) = pInteger->Sar(*pRightInteger);
		break;
	case(Operation::Sal) :
		(*pResultInteger) = pInteger->Sal(*pRightInteger);
		break;
	case(Operation::BitXor) :
		(*pResultInteger) = pInteger->BitXor(*pRightInteger);
		break;
	case(Operation::BitAnd) :
		(*pResultInteger) = pInteger->BitAnd(*pRightInteger);
		break;
	case(Operation::BitOr) :
		(*pResultInteger) = pInteger->BitOr(*pRightInteger);
		break;
	default:
		break;
	}
	return ivValue;
}

IrisValue IrisInteger::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisInteger::Add(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Add, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Sub(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Sub, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Mul(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Mul, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Div(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Div, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Power(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Power, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Mod(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Mod, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Shl(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return CastOperation(Operation::Shl, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Shr(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::Shr, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Sar(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::Sar, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::Sal(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::Sal, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::BitXor(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::BitXor, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::BitAnd(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::BitAnd, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::BitOr(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return Operate(Operation::BitOr, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
}

IrisValue IrisInteger::BitNot(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivValue;
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	// 新建临时Integer对象作为结果
	//ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
	ivValue = IrisDevUtil::CreateInt(0);
	// 获取新建对象的Native Pointer
	IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
	(*pResultInteger) = pInteger->BitNot();
	return ivValue;
}

IrisValue IrisInteger::Equal(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::Equal, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::NotEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::NotEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::BigThan(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThan, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::BigThanOrEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::BigThanOrEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::LessThan(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThan, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::LessThanOrEqual(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValuse, IIrisContextEnvironment * pContextEnvironment) {
	return CmpOperation(Operation::LessThanOrEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
}

IrisValue IrisInteger::Plus(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateInt(pInteger->m_nInteger);
	return ivValue;
}

IrisValue IrisInteger::Minus(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	IrisValue ivValue = IrisDevUtil::CreateInt(-pInteger->m_nInteger);
	return ivValue;
}

IrisValue IrisInteger::ToString(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
	return IrisDevUtil::CreateString(pInteger->ToString().c_str());
}

IrisValue IrisInteger::ToFloat(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto nInteger = IrisDevUtil::GetInt(ivObj);
	return IrisDevUtil::CreateFloat(static_cast<int>(nInteger));
}

IrisInteger::IrisInteger()
{
}


IrisInteger::~IrisInteger()
{
}
