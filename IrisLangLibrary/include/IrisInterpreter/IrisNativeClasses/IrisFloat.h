#ifndef _H_IRISFLOAT_
#define _H_IRISFLOAT_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisFloatTag.h"
#include "IrisInterpreter.h"

class IrisFloat : public IIrisClass
{
private:

	enum class Operation {
		Add = 0,
		Sub,
		Mul,
		Div,
		Power,

		Equal,
		NotEqual,
		BigThan,
		BigThanOrEqual,
		LessThan,
		LessThanOrEqual,
	};

	static IrisValue CmpOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment){
		IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
		IrisFloatTag iftRightFloat;
		// 如果右边为Integer，则转化为Float之间的运算
		if (IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Integer")) {
			// 获取右边的值
			IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
			// 将右边对象转换为Float
			iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
		}
		else {
			if (IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Float")) {
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

	static IrisValue CastOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivValue;
		IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
		IrisFloatTag iftRightFloat;
		// 如果右边为Integer，则转化为Float之间的运算
		if (IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "Integer")) {
			// 获取右边的值
			IrisIntegerTag* pRightInteger = IrisDevUtil::GetNativePointer<IrisIntegerTag*>((IrisValue&)ivsValues->GetValue(0));
			// 将右边对象转换为Float
			iftRightFloat = static_cast<IrisFloatTag>(*pRightInteger);
		}
		else {
			iftRightFloat = *(IrisDevUtil::GetNativePointer<IrisFloatTag*>((IrisValue&)ivsValues->GetValue(0)));
		}
		// 新建临时Float对象作为结果
		ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Float"), nullptr, pContextEnvironment);
		// 获取新建对象的Native Pointer
		IrisFloatTag* pResultFloat = static_cast<IrisFloatTag*>(ivValue.GetInstanceNativePointer());
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

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::Equal, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::NotEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BigThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::BigThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue BigThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::BigThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue LessThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::LessThan, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue LessThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return CmpOperation(Operation::LessThanOrEqual, ivObj, ivsValues, ivsVariableValues, pContextEnvironment);
	}

	static IrisValue Plus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
		IrisValue ivValue = IrisDevUtil::CreateFloat(pFloat->m_dFloat);
		return ivValue;
	}

	static IrisValue Minus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
		IrisValue ivValue = IrisDevUtil::CreateFloat(-pFloat->m_dFloat);
		return ivValue;
	}

	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisFloatTag* pFloat = IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivObj);
		return IrisDevUtil::CreateString(pFloat->ToString().c_str());
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Float";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisFloatTag);
	}

	void* NativeAlloc() {
		return new IrisFloatTag(0.0);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisFloatTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "+", Add, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "-", Sub, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "*", Mul, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "/", Div, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "**", Power, 1, false);

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
	//	return new IrisFloatTag(szLiterral);
	//}

	static double GetFloatData(IrisValue& ivValue) {
		return IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivValue)->m_dFloat;
	}

	//IrisValue CreateInstanceByInstantValue(char* szLiterral){
	//	IrisValue ivValue;
	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	IrisFloatTag* pFloat = new IrisFloatTag(szLiterral);
	//	pObject->SetNativeObject(pFloat);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	//	return ivValue;
	//}

	//IrisValue CreateInstanceByInstantValue(double dValue){
	//	IrisValue ivValue;
	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	IrisFloatTag* pFloat = new IrisFloatTag(dValue);
	//	pObject->SetNativeObject(pFloat);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	//	return ivValue;
	//}

	IrisFloat();
	~IrisFloat();
};

#endif