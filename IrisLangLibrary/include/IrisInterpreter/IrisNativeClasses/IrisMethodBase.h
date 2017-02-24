#ifndef _H_IRISMETHODBASE_
#define _H_IRISMETHODBASE_

#include "IrisDevHeader.h"
#include "IrisMethodBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisString.h"

class IrisMethodBase : public IIrisClass
{

public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue GetMethodName(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Method";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "method_name", GetMethodName, 0, false);
	}

	IrisMethodBase();
	~IrisMethodBase();
};

#endif