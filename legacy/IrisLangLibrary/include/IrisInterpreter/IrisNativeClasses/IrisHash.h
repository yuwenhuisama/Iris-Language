#ifndef _H_IRISHASH_
#define _H_IRISHASH_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisHashTag.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"

class IrisHash : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue At(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue Set(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue Each(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue Size(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue GetIterator(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);

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
			static_cast<IrisObject*>(((IrisValue&)pair.first).GetIrisObject())->Mark();
			static_cast<IrisObject*>(((IrisValue&)pair.second).GetIrisObject())->Mark();
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