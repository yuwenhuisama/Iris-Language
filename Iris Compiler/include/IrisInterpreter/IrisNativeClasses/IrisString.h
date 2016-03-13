#ifndef _H_IRISSTRING_
#define _H_IRISSTRING_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisStringTag.h"

class IrisString : public IIrisClass
{
public:

	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue Add(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment){
		IrisValue ivValue;
		IrisStringTag* pString = IrisDev_GetNativePointer<IrisStringTag*>(ivObj);
		if (!IrisDev_CheckClass((IrisValue&)ivsValues->GetValue(0), "String")) {
			IrisDev_GroanIrregularWithString("String CAN ONLY be added with a String object.");
			return IrisDev_Nil();
		}
		IrisStringTag* pAddedString = IrisDev_GetNativePointer<IrisStringTag*>((IrisValue&)ivsValues->GetValue(0));
		ivValue = IrisDev_CreateInstance(IrisDev_GetClass("String"), nullptr, pContextEnvironment);
		IrisStringTag* pResultString = IrisDev_GetNativePointer<IrisStringTag*>(ivValue);
		(*pResultString) = pString->Add(*pAddedString);
		return ivValue;
	}

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pSelf = IrisDev_GetNativePointer<IrisStringTag*>(ivObj);
		auto& ivTarget = (IrisValue&)ivsValues->GetValue(0);
		if (!IrisDev_CheckClass(ivTarget, "String")) {
			return IrisDev_False();
		}
		auto pTarget = IrisDev_GetNativePointer<IrisStringTag*>(ivTarget);
		return pSelf->GetString() == pTarget->GetString() ? IrisDev_True() : IrisDev_False();
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "String";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	static const string& GetString(IrisValue& ivValue) {
		return IrisDev_GetNativePointer<IrisStringTag*>(ivValue)->GetString();
	}

	IrisString();

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisStringTag);
	}

	void* NativeAlloc() {
		return new IrisStringTag("");
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisStringTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "+", Add, 1, false);
		IrisDev_AddInstanceMethod(this, "==", Equal, 1, false);
	}

	//IrisValue CreateInstanceByInstantValue(const string& strString) {
	//	IrisValue ivValue;
	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	IrisStringTag* pString = new IrisStringTag(strString);
	//	pObject->SetNativeObject(pString);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	//	return ivValue;
	//}

	~IrisString();
};

#endif