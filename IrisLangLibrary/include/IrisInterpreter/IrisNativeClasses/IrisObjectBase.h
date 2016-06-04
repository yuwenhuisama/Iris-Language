#ifndef _H_IRISOBJECTBASE_
#define _H_IRISOBJECTBASE_

#include "IrisDevHeader.h"
#include "IrisStringTag.h"
#include "IrisIntegerTag.h"

#include "IrisInterpreter/IrisNativeModules/IrisKernel.h"

class IrisObjectBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetClass(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetObjectID(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LogicOr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue LogicAnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Fix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Unfix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue IsFixed(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Object";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	// Ãÿ ‚¥¶¿Ì
	void* NativeAlloc() {
		return nullptr;
	}

	void NativeFree(void* pNativePointer) {
		return;
	}

	int GetTrustteeSize(void* pNativePointer) {
		return 0;
	}

	void NativeClassDefine() {

		IrisDevUtil::AddModule(this, IrisDevUtil::GetModule("Kernel"));

		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "to_string", ToString, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "==", Equal, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "!=", NotEqual, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "&&", LogicAnd, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "||", LogicOr, 1, false);

		IrisDevUtil::AddInstanceMethod(this, "unfix", Unfix, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "fix", Fix, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "fixed", IsFixed, 0, false);

		IrisDevUtil::AddGetter(this, "@__class", GetClass);
		IrisDevUtil::AddGetter(this, "@object_id", GetObjectID);

	}

	IrisObjectBase();
	~IrisObjectBase();
};

#endif