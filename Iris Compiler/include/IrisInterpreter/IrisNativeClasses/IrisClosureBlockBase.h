#ifndef _H_IRISCLOSUREBLOCKBASE_
#define _H_IRISCLOSUREBLOCKBASE_

#include "IrisDevHeader.h"

#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisClosureBlockBaseTag.h"

class IrisClosureBlockBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {

		IIrisClosureBlock* pClosureBlock = nullptr;
		pClosureBlock = pContextEnvironment->GetClosureBlock();
		IrisDev_GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->SetClosureBlock(static_cast<IrisClosureBlock*>(pClosureBlock));
		pContextEnvironment->SetClosureBlock(nullptr);

		return ivObj;
	}

	static IrisValue Call(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_GetNativePointer<IrisClosureBlockBaseTag*>(ivObj)->GetClosureBlock()->Excute(ivsVariableValues);
	}

public:

	const char* NativeClassNameDefine() const {
		return "Block";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	void Mark(void* pNativeObjectPointer) {
		IrisClosureBlockBaseTag* pClosureBlock = static_cast<IrisClosureBlockBaseTag*>(pNativeObjectPointer);
		pClosureBlock->GetClosureBlock()->Mark();
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisClosureBlockBaseTag);
	}

	void* NativeAlloc() {
		return new IrisClosureBlockBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisClosureBlockBaseTag*>(pNativePointer)->GetClosureBlock();
		delete static_cast<IrisClosureBlockBaseTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDev_AddInstanceMethod(this, "call", Call, 0, true);
	}

	IrisClosureBlockBase();
	~IrisClosureBlockBase();
};

#endif