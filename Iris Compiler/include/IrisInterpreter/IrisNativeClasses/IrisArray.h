#ifndef _H_IRISARRAY_
#define _H_IRISARRAY_

#include "IrisDevHeader.h"

#include "IrisArrayTag.h"
#include "IrisInteger.h"

class IrisArray : public IIrisClass
{

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
		return ivObj;
	}

	static IrisValue At(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivIndex = (IrisValue&)(ivsValues->GetValue(0)); //(*ivsValues)[0];
		if (!IrisDev_CheckClass(ivIndex, "Integer")) {
			IrisDev_GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
			return IrisDev_Nil();
		}
		return IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->At(IrisInteger::GetIntData(ivIndex));
	}

	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValue& ivIndex = (IrisValue&)(ivsValues->GetValue(0));
		if (!IrisDev_CheckClass(ivIndex, "Integer")) {
			IrisDev_GroanIrregularWithString("The index of an ARRAY object must be an INTEGER.");
			return IrisDev_Nil();
		}
		IrisValue& ivValue = (IrisValue&)(ivsValues->GetValue(1));
		return IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->Set(IrisInteger::GetIntData(ivIndex), ivValue);
	}

	static IrisValue Each(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pClosureBlock = pContextEnvironment->GetClosureBlock();

		if (!pClosureBlock) {
			IrisDev_GroanIrregularWithString("The method of each of ARRAY needs a block.");
			return IrisDev_Nil();
		}

		IrisValues ivValues(1);
		for (auto& elem : IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->m_vcValues) {
			ivValues[0] = elem;
			pClosureBlock->Excute(&ivValues);
			if (IrisDev_IrregularHappened() || IrisDev_FatalErrorHappened()) {
				break;
			}
		}
		return IrisDev_Nil();
	}

	static IrisValue Push(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		if (ivObj.GetIrisObject()->IsFixed()) {
			IrisDev_GroanIrregularWithString("Cannot push value into a fixed ARRAY object.");
			return IrisDev_Nil();
		}
		return IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->Push((ivsValues->GetValue(0)));
	}

	static IrisValue Pop(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		if (ivObj.GetIrisObject()->IsFixed()) {
			IrisDev_GroanIrregularWithString("Cannot pop value from a fixed ARRAY object.");
			return IrisDev_Nil();
		}
		return IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->Pop();
	}

	static IrisValue Size(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev_CreateInt(IrisDev_GetNativePointer<IrisArrayTag*>(ivObj)->Size());
	}

	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValues ivsParameter = { ivObj };
		return IrisDev_CreateInstance(IrisDev_GetClass("ArrayIterator"), &ivsParameter, pContextEnvironment);
	}

public:

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisArrayTag);
	}

	void Mark(void* pNativeObjectPointer) {
		IrisArrayTag* pArray = static_cast<IrisArrayTag*>(pNativeObjectPointer);
		for (auto value : pArray->m_vcValues) {
			value.GetIrisObject()->Mark();
		}
	}

	void* NativeAlloc() {
		return new IrisArrayTag();
	}

	void NativeFree(void* pNativePointer){
		delete static_cast<IrisArrayTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, true);
		IrisDev_AddInstanceMethod(this, "[]", At, 1, false);
		IrisDev_AddInstanceMethod(this, "[]=", Set, 2, false);

		IrisDev_AddInstanceMethod(this, "push", Push, 1, false);
		IrisDev_AddInstanceMethod(this, "pop", Pop, 0, false);
		IrisDev_AddInstanceMethod(this, "each", Each, 0, false);

		IrisDev_AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);

		IrisDev_AddGetter(this, "@size", Size);
	}

	const char* NativeClassNameDefine() const {
		return "Array";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	IrisArray();
	~IrisArray();
};

#endif