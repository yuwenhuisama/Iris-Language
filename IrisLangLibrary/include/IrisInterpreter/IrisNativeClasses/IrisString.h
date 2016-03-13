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
		IrisStringTag* pString = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivObj);
		if (!IrisDevUtil::CheckClass((IrisValue&)ivsValues->GetValue(0), "String")) {
			IrisDevUtil::GroanIrregularWithString("String CAN ONLY be added with a String object.");
			return IrisDevUtil::Nil();
		}
		IrisStringTag* pAddedString = IrisDevUtil::GetNativePointer<IrisStringTag*>((IrisValue&)ivsValues->GetValue(0));
		ivValue = IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("String"), nullptr, pContextEnvironment);
		IrisStringTag* pResultString = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivValue);
		(*pResultString) = pString->Add(*pAddedString);
		return ivValue;
	}

	static IrisValue Equal(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pSelf = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivObj);
		auto& ivTarget = (IrisValue&)ivsValues->GetValue(0);
		if (!IrisDevUtil::CheckClass(ivTarget, "String")) {
			return IrisDevUtil::False();
		}
		auto pTarget = IrisDevUtil::GetNativePointer<IrisStringTag*>(ivTarget);
		return pSelf->GetString() == pTarget->GetString() ? IrisDevUtil::True() : IrisDevUtil::False();
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "String";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	static const string& GetString(IrisValue& ivValue) {
		return IrisDevUtil::GetNativePointer<IrisStringTag*>(ivValue)->GetString();
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "+", Add, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "==", Equal, 1, false);
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