#ifndef _H_IRISINTERFACEBASE_
#define _H_IRISINTERFACEBASE_

#include "IrisDevHeader.h"
#include "IrisInterfaceBaseTag.h"
#include "IrisString.h"

class IrisInterfaceBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetInterfaceName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Interface";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisInterfaceBaseTag);
	}

	void* NativeAlloc() {
		return new IrisInterfaceBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisInterfaceBaseTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "interface_name", GetInterfaceName, 0, false);
	}

	IrisInterfaceBase();
	~IrisInterfaceBase();
};

#endif