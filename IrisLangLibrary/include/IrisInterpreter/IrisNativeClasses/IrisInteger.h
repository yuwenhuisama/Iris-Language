#ifndef _H_IRISINTEGER_
#define _H_IRISINTEGER_
#include "IrisDevHeader.h"

#include "IrisIntegerTag.h"
#include "IrisFloatTag.h"

class IrisInteger : public IIrisClass
{
private:

	enum class Operation {
		Add = 0,
		Sub,
		Mul,
		Div,
		Power,
		Mod,

		Shr,
		Shl,
		Sar,
		Sal,
		BitXor,
		BitAnd,
		BitOr,

		Equal,
		NotEqual,
		BigThan,
		BigThanOrEqual,
		LessThan,
		LessThanOrEqual,
	};

	static IrisValue CmpOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		bool bResult = false;
		// 如果右边为Float，则转化为Float之间的运算
		if (IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Float")) {
			// 获取被加数
			IrisFloatTag* pRightFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>((IrisValue&)ivsValues->GetValue(0));
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
			if (!IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Integer")) {
				IrisDevUtil::GroanIrregularWithString("Invaid parameter was sent.");
				return IrisDevUtil::Nil();
			}
			// 获取被加数
			IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
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

	static IrisValue CastOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivValue;
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		// 如果右边为Float，则转化为Float之间的运算
		if (IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Float")) {
			// 获取被加数
			IrisFloatTag* pRightFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>((IrisValue&)ivsValues->GetValue(0));
			// 将当前对象转换为Float
			IrisFloatTag& iftLeftFloat = static_cast<IrisFloatTag>(*pInteger);
			if (eOperationType != Operation::Mod) {
				// 新建临时Float对象作为结果
				ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Float"), nullptr, pContextEnvironment);
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
				ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
				// 获取新建对象的Native Pointer
				IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
				(*pResultInteger) = pInteger->Mod(static_cast<IrisIntegerTag>(*pRightFloat));
			}
		}
		else {
			// 获取被加数
			IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
			// 新建临时Integer对象作为结果
			ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
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

	static IrisValue Operation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivValue;
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		// 获取Right
		IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
		// 新建临时Integer对象作为结果
		ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
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

public:

	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue Add(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Add, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Sub(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Sub, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Mul(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Mul, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Div(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Div, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Power(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Power, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}
	static IrisValue Mod(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Mod, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Shl(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CastOperation(Operation::Shl, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Shr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::Shr, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Sar(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::Sar, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Sal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::Sal, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BitXor(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::BitXor, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BitAnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::BitAnd, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BitOr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return Operation(Operation::BitOr, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BitNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivValue;
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		// 新建临时Integer对象作为结果
		ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Integer"), nullptr, pContextEnvironment);
		// 获取新建对象的Native Pointer
		IrisIntegerTag* pResultInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue);
		(*pResultInteger) = pInteger->BitNot();
		return ivValue;
	}

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::Equal, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::NotEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue BigThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::BigThan, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue BigThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::BigThanOrEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue LessThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::LessThan, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue LessThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::LessThanOrEqual, ivObj, ivsValues, ivsVariableValuse, pContextEnvironment);
	}

	static IrisValue Plus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		IrisValue ivValue = IrisDevUtil::CreateInt(pInteger->m_nInteger);
		return ivValue;
	}

	static IrisValue Minus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		IrisValue ivValue = IrisDevUtil::CreateInt(-pInteger->m_nInteger);
		return ivValue;
	}

	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisIntegerTag* pInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivObj);
		return IrisDevUtil::CreateString(pInteger->ToString().c_str());
	}

public:
	IrisInteger();

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Integer";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisIntegerTag);
	}

	void* NativeAlloc() {
		return new IrisIntegerTag(0);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisIntegerTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "+", Add, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "-", Sub, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "*", Mul, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "/", Div, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "**", Power, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "<<", Shl, 1, false);
		IrisDevUtil::AddInstanceMethod(this, ">>", Shr, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "<<<", Sal, 1, false);
		IrisDevUtil::AddInstanceMethod(this, ">>>", Sar, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "^", BitXor, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "&", BitAnd, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "|", BitOr, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "~", BitNot, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "%", Mod, 1, false);

		IrisDevUtil::AddInstanceMethod(this, "==", Equal, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "!=", NotEqual, 1, false);
		IrisDevUtil::AddInstanceMethod(this, ">", BigThan, 1, false);
		IrisDevUtil::AddInstanceMethod(this, ">=", BigThanOrEqual, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "<", LessThan, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "<=", LessThanOrEqual, 1, false);

		IrisDevUtil::AddInstanceMethod(this, "__plus", Plus, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "__minus", Minus, 0, false);

		IrisDevUtil::AddInstanceMethod(this, "to_string", ToString, 0, false);
	}

	//virtual void* GetLiteralObject(char* szLiterral) {
	//	return new IrisIntegerTag(szLiterral);
	//}

	static int GetIntData(IrisValue& ivValue) {
		return IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivValue)->m_nInteger;
	}

	//IrisValue CreateInstanceByInstantValue(char* szLiterral) {
	//	IrisValue ivValue;
	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	IrisIntegerTag* pInteger = new IrisIntegerTag(szLiterral);
	//	pObject->SetNativeObject(pInteger);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	//	return ivValue;
	//}

	//IrisValue CreateInstanceByInstantValue(int nValue) {
	//	IrisValue ivValue;
	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	IrisIntegerTag* pInteger = new IrisIntegerTag(nValue);
	//	pObject->SetNativeObject(pInteger);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	//	return ivValue;
	//}

	~IrisInteger();

	friend class IrisFloatTag;
};

#endif