#ifndef _H_IRISARRAY_
#define _H_IRISARRAY_

#include "IrisDevHeader.h"
#include "IrisArrayTag.h"
#include "IrisInteger.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"

class IrisArrayTag;

class IrisArray : public IIrisClass
{

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue At(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Each(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Push(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Pop(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Size(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetIterator(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

	static IrisValue Empty(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue IndexOf(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Include(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue Merge(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

	static IrisValue Sort(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	


public:

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisArrayTag);
	}

	void Mark(void* pNativeObjectPointer) {
		IrisArrayTag* pArray = static_cast<IrisArrayTag*>(pNativeObjectPointer);
		for (auto value : pArray->m_vcValues) {
			IrisDevUtil::MarkObject(value);
		}
	}

	void* NativeAlloc() {
		return new IrisArrayTag();
	}

	void NativeFree(void* pNativePointer){
		delete static_cast<IrisArrayTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, true);
		IrisDevUtil::AddInstanceMethod(this, "[]", At, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "[]=", Set, 2, false);

		IrisDevUtil::AddInstanceMethod(this, "push", Push, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "pop", Pop, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "each", Each, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "sort", Sort, 0, false);

		IrisDevUtil::AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);

		IrisDevUtil::AddGetter(this, "@size", Size);

	}

	const char* NativeClassNameDefine() const {
		return "Array";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	IrisArray();
	~IrisArray();
};

#endif