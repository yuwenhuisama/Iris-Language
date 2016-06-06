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

	static IrisValue CmpOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue CastOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Operate(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Add(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sub(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Mul(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Div(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Power(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Mod(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Shl(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Shr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sar(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitXor(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitAnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitOr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitNot(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Plus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Minus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToFloat(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

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
		IrisDevUtil::AddInstanceMethod(this, "to_float", ToFloat, 0, false);
	}

	//virtual void* GetLiteralObject(char* szLiterral) {
	//	return new IrisIntegerTag(szLiterral);
	//}

	static int GetIntData(const IrisValue& ivValue) {
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