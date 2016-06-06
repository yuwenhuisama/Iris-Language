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

	static IrisValue CmpOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue CastOperation(Operation eOperationType, IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Add(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Sub(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Mul(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Div(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Power(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue BigThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThan(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LessThanOrEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Plus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Minus(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToInteger(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

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
		IrisDevUtil::AddInstanceMethod(this, "to_integer", ToInteger, 0, false);
	}

	static double GetFloatData(const IrisValue& ivValue) {
		return IrisDevUtil::GetNativePointer<IrisFloatTag*>(ivValue)->m_dFloat;
	}

	IrisFloat();
	~IrisFloat();
};

#endif