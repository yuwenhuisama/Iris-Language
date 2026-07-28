#ifndef _H_IRISHASHITERATOR_
#define _H_IRISHASHITERATOR_

#include "IrisDevHeader.h"

#include "IrisHashTag.h"
#include "IrisInteger.h"
#include "IrisHashIteratorTag.h"
#include "IrisInterpreter.h"

class IrisHashIterator :
	public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue Next(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue IsEnd(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue GetKey(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue GetValue(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);

public:

	void Mark(void* pNativeObjectPointer) {}
	const char* NativeClassNameDefine() const {
		return "HashIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisHashIteratorTag);
	}

	void* NativeAlloc() {
		return new IrisHashIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete (IrisHashIteratorTag*)pNativePointer;
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "next", Next, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_key", GetKey, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}

	IrisHashIterator();
	~IrisHashIterator();
};

#endif