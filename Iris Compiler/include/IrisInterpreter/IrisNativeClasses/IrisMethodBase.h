#ifndef _H_IRISMETHODBASE_
#define _H_IRISMETHODBASE_

#include "IrisDevHeader.h"
#include "IrisMethodBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisString.h"

class IrisMethodBase : public IIrisClass
{

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetMethodName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisMethodBaseTag* pMethod = IrisDev_GetNativePointer<IrisMethodBaseTag*>(ivObj);
		const string& strMethodName = pMethod->GetMethodName();
		return IrisDev_CreateString(strMethodName.c_str());
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Method";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisMethodBaseTag);
	}

	void* NativeAlloc() {
		return new IrisMethodBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		delete (static_cast<IrisMethodBaseTag*>(pNativePointer))->GetMethod();
		delete static_cast<IrisMethodBaseTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "method_name", GetMethodName, 0, false);
	}

	IrisMethodBase();
	~IrisMethodBase();
};

#endif