#ifndef _H_IRISSTRING_
#define _H_IRISSTRING_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisStringTag.h"

class IrisString : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue Add(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue Equal(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "String";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	static const string& GetString(const IrisValue& ivValue) {
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