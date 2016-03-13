#ifndef _H_IRISHASH_
#define _H_IRISHASH_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisHashTag.h"

class IrisHash : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Initialize(static_cast<IrisValues*>(ivsVariableValues));
		return ivObj;
	}

	static IrisValue At(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivIndex = ivsValues->GetValue(0);
		return IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->At(ivIndex);
	}

	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		const IrisValue& ivIndex = ivsValues->GetValue(0);
		const IrisValue& ivValue = ivsValues->GetValue(1);
		return IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Set(ivIndex, ivValue);
	}

	static IrisValue Each(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto pClosureBlock = pContextEnvironment->GetClosureBlock();
		IrisValues ivValues(2);
		for (auto& elem : IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->m_mpHash) {
			ivValues[0] = elem.first;
			ivValues[1] = elem.second;
			pClosureBlock->Excute(&ivValues);
			if (IrisDevUtil::IrregularHappened() || IrisDevUtil::FatalErrorHappened()) {
				break;
			}
		}
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}

	static IrisValue Size(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDevUtil::CreateInt(IrisDevUtil::GetNativePointer<IrisHashTag*>(ivObj)->Size());
	}

	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisValues ivsParameter = { ivObj };
		return IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("HashIterator"), &ivsParameter, pContextEnvironment);
	}

public:

	const char* NativeClassNameDefine() const {
		return "Hash";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
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
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, true);
		IrisDevUtil::AddInstanceMethod(this, "[]", At, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "[]=", Set, 2, false);
		IrisDevUtil::AddInstanceMethod(this, "each", Each, 0, false);

		IrisDevUtil::AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);

		IrisDevUtil::AddGetter(this, "@size", Size);
	}

	IrisHash();
	~IrisHash();
};

#endif