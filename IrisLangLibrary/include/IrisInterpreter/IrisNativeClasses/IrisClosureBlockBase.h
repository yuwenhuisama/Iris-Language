#ifndef _H_IRISCLOSUREBLOCKBASE_
#define _H_IRISCLOSUREBLOCKBASE_

#include "IrisDevHeader.h"

#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisClosureBlockBaseTag.h"

class IrisClosureBlockBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Call(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	const char* NativeClassNameDefine() const {
		return "Block";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	void Mark(void* pNativeObjectPointer) {
		IrisClosureBlockBaseTag* pClosureBlock = static_cast<IrisClosureBlockBaseTag*>(pNativeObjectPointer);
		IrisDevUtil::MarkClosureBlock(pClosureBlock->GetClosureBlock());
		//static_cast<IrisClosureBlock*>(pClosureBlock->GetClosureBlock())->GetNativeObject()->Mark();
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisClosureBlockBaseTag);
	}

	void* NativeAlloc() {
		return new IrisClosureBlockBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		if (pNativePointer) {
			delete static_cast<IrisClosureBlockBaseTag*>(pNativePointer)->GetClosureBlock();
			delete static_cast<IrisClosureBlockBaseTag*>(pNativePointer);
		}
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "call", Call, 0, true);
	}

	IrisClosureBlockBase();
	~IrisClosureBlockBase();
};

#endif