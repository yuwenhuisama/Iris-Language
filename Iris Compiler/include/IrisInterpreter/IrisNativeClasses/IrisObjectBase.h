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
		return IrisDev_CreateInt(nObjectID);
	}

	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const string& strClassName = ivObj.GetIrisObject()->GetClass()->GetInternClass()->GetClassName();
		IrisValue ivStringObjectID = IrisDev_CallMethod(ivObj, nullptr, "__get_object_id");
		const string& strObjectID = IrisDev_GetNativePointer<IrisIntegerTag*>(ivStringObjectID)->ToString();
		string strOutString = "<" + strClassName + ":" + strObjectID + ">";
		return IrisDev_CreateString(strOutString.c_str());
	}

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivDestObj = ivsValues->GetValue(0);
		if (ivObj == ivDestObj){
			return IrisDev_True();
		}
		else {
			return IrisDev_False();
		}
	}

	static IrisValue NotEqual(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue ivResult = IrisDev_CallMethod(ivObj, ivsValues, "==");
			//ivObj.GetIrisObject()->CallInstanceFunction("==", pContextEnvironment, ivsValues, CallerSide::Outside);
		if (ivResult == IrisDev_True()) {
			return IrisDev_False();
		}
		else{
			return IrisDev_True();
		}
	}

	static IrisValue LogicOr(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivDest = (IrisValue&)ivsValues->GetValue(0);
		// 左右两边只要有一个不为nil或false那么就为true
		if (ivObj != IrisDev_False() && ivObj != IrisDev_Nil()) {
			return IrisDev_True();
		}
		else if (ivDest != IrisDev_False() && ivDest != IrisDev_Nil()){
			return IrisDev_True();
		}
		else {
			return IrisDev_False();
		}
	}

	static IrisValue LogicAnd(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivDest = (IrisValue&)ivsValues->GetValue(0);
		// 左右两边只要有一个为nil或false那么就为false
		if (ivObj == IrisDev_False() || ivObj == IrisDev_Nil()) {
			return IrisDev_False();
		}
		else if (ivDest == IrisInterpreter::CurrentInterpreter()->False() && ivDest == IrisInterpreter::CurrentInterpreter()->Nil()){
			return IrisDev_False();
		}
		else {
			return IrisDev_True();
		}
	}

	static IrisValue Fix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		pObject->Fix();
		return IrisDev_Nil();
	}

	static IrisValue Unfix(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		pObject->Unfix();
		return IrisDev_Nil();
	}

	static IrisValue IsFixed(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pObject = ivObj.GetIrisObject();
		return pObject->IsFixed() ? IrisDev_True() : IrisDev_False();;
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

		IrisDev_AddModule(this, IrisDev_GetModule("Kernel"));

		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "to_string", ToString, 0, false);
		IrisDev_AddInstanceMethod(this, "==", Equal, 1, false);
		IrisDev_AddInstanceMethod(this, "!=", NotEqual, 1, false);
		IrisDev_AddInstanceMethod(this, "&&", LogicAnd, 1, false);
		IrisDev_AddInstanceMethod(this, "||", LogicOr, 1, false);

		IrisDev_AddInstanceMethod(this, "unfix", Unfix, 0, false);
		IrisDev_AddInstanceMethod(this, "fix", Fix, 0, false);
		IrisDev_AddInstanceMethod(this, "fixed", IsFixed, 0, false);

		IrisDev_AddGetter(this, "@__class", GetClass);
		IrisDev_AddGetter(this, "@object_id", GetObjectID);

	}

	IrisObjectBase();
	~IrisObjectBase();
};

#endif