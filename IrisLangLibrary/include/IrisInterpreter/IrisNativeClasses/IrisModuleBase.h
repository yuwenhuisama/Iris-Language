#ifndef _H_IRISMODULEBASE_
#define _H_IRISMODULEBASE_

#include "IrisDevHeader.h"
#include "IrisModuleBaseTag.h"
#include "IrisString.h"

class IrisModuleBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetModuleName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Module";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisModuleBaseTag);
	}

	void* NativeAlloc() {
		return new IrisModuleBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisModuleBase*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "module_name", GetModuleName, 0, false);
	}

	IrisModuleBase();
	~IrisModuleBase();
};

#endif