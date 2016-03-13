#ifndef _H_IRISHASH_
#define _H_IRISHASH_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisHashTag.h"

class IrisHash : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDev_GetNativePointer<IrisHashTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
		return ivObj;
	}

	static IrisValue At(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivIndex = ivsValues->GetValue(0);
		return IrisDev_GetNativePointer<IrisHashTag*>(ivObj)->At(ivIndex);
	}

	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivIndex = ivsValues->GetValue(0);
		const IrisValue& ivValue = ivsValues->GetValue(1);
		return IrisDev_GetNativePointer<IrisHashTag*>(ivObj)->Set(ivIndex, ivValue);
	}

	static IrisValue Each(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pClosureBlock = pContextEnvironment->GetClosureBlock();
		IrisValues ivValues(2);
		for (auto& elem : IrisDev_GetNativePointer<IrisHashTag*>(ivObj)->m_mpHash) {
			ivValues[0] = elem.first;
			ivValues[1] = elem.second;
			pClosureBlock->Excute(&ivValues);
			if (IrisDev_IrregularHappened() || IrisDev_FatalErrorHappened()) {
				break;
			}
		}
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}

	static IrisValue Size(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_CreateInt(IrisDev_GetNativePointer<IrisHashTag*>(ivObj)->Size());
	}

	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValues ivsParameter = { ivObj };
		return IrisDev_CreateInstance(IrisDev_GetClass("HashIterator"), &ivsParameter, pContextEnvironment);
	}

public:

	const char* NativeClassNameDefine() const {
		return "Hash";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	void Mark(void* pNativeObjectPointer) {
		IrisHashTag* pHash = (IrisHashTag*)pNativeObjectPointer;
		for (auto& pair : pHash->m_mpHash) {
			((IrisValue&)pair.first).GetIrisObject()->Mark();
			pair.second.GetIrisObject()->Mark();
		}
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisHashTag);
	}

	void* NativeAlloc() {
		return new IrisHashTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisHashTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, true);
		IrisDev_AddInstanceMethod(this, "[]", At, 1, false);
		IrisDev_AddInstanceMethod(this, "[]=", Set, 2, false);
		IrisDev_AddInstanceMethod(this, "each", Each, 0, false);

		IrisDev_AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);

		IrisDev_AddGetter(this, "@size", Size);
	}

	IrisHash();
	~IrisHash();
};

#endif