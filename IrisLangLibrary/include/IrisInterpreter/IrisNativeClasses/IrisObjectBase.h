#ifndef _H_IRISOBJECTBASE_
#define _H_IRISOBJECTBASE_

#include "IrisDevHeader.h"
#include "IrisStringTag.h"
#include "IrisIntegerTag.h"

#include "IrisInterpreter/IrisNativeModules/IrisKernel.h"

class IrisObjectBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetClass(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IIrisObject* pObject = ivObj.GetIrisObject()->GetClass()->GetInternClass()->GetClassObject();
		return IrisValue::WrapObjectPointerToIrisValue(pObject);
	}

	static IrisValue GetObjectID(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IIrisObject* pObject = ivObj.GetIrisObject();
		int nObjectID = pObject->GetObjectID();
		return IrisDevUtil::CreateInt(nObjectID);
	}

	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const string& strClassName = ivObj.GetIrisObject()->GetClass()->GetInternClass()->GetClassName();
		IrisValue ivStringObjectID = IrisDevUtil::CallMethod(ivObj, nullptr, "__get_object_id");
		const string& strObjectID = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivStringObjectID)->ToString();
		string strOutString = "<" + strClassName + ":" + strObjectID + ">";
		return IrisDevUtil::CreateString(strOutString.c_str());
	}

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivDestObj = ivsValues->GetValue(0);
		if (ivObj == ivDestObj){
			return IrisDevUtil::True();
		}
		else {
			return IrisDevUtil::False();
		}
	}

	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivResult = IrisDevUtil::CallMethod(ivObj, ivsValues, "==");
			//ivObj.GetIrisObject()->CallInstanceFunction("==", pContextEnvironment, ivsValues, CallerSide::Outside);
		if (ivResult == IrisDevUtil::True()) {
			return IrisDevUtil::False();
		}
		else{
			return IrisDevUtil::True();
		}
	}

	static IrisValue LogicOr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivDest = (IrisValue&)ivsValues->GetValue(0);
		// 左右两边只要有一个不为nil或false那么就为true
		if (ivObj != IrisDevUtil::False() && ivObj != IrisDevUtil::Nil()) {
			return IrisDevUtil::True();
		}
		else if (ivDest != IrisDevUtil::False() && ivDest != IrisDevUtil::Nil()){
			return IrisDevUtil::True();
		}
		else {
			return IrisDevUtil::False();
		}
	}

	static IrisValue LogicAnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivDest = (IrisValue&)ivsValues->GetValue(0);
		// 左右两边只要有一个为nil或false那么就为false
		if (ivObj == IrisDevUtil::False() || ivObj == IrisDevUtil::Nil()) {
			return IrisDevUtil::False();
		}
		else if (ivDest == IrisInterpreter::CurrentInterpreter()->False() && ivDest == IrisInterpreter::CurrentInterpreter()->Nil()){
			return IrisDevUtil::False();
		}
		else {
			return IrisDevUtil::True();
		}
	}

	static IrisValue Fix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		pObject->Fix();
		return IrisDevUtil::Nil();
	}

	static IrisValue Unfix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		pObject->Unfix();
		return IrisDevUtil::Nil();
	}

	static IrisValue IsFixed(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		return pObject->IsFixed() ? IrisDevUtil::True() : IrisDevUtil::False();;
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Object";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	// 特殊处理
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