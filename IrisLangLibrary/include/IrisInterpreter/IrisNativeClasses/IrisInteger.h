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

	static IrisValue CmpOperation(Operation eOperationType, const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue CastOperation(Operation eOperationType, const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Operate(Operation eOperationType, const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	static IrisValue InitializeFunction(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Add(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sub(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Mul(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Div(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Power(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Mod(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Shl(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Shr(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sar(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sal(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitXor(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitAnd(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitOr(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BitNot(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Equal(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue NotEqual(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThan(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThanOrEqual(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThan(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThanOrEqual(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValuse, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Plus(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Minus(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToString(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToFloat(const IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

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