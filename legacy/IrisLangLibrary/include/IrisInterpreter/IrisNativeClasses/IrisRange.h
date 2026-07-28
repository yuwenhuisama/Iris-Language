#ifndef _H_IRISRANGE_
#define _H_IRISRANGE_

#include "IrisDevHeader.h"

#include "IrisRangeTag.h"
#include "IrisInteger.h"

class IrisRange : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue GetIterator(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);

public:
	
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Range";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisRangeTag);
	}

	void* NativeAlloc() {
		return new IrisRangeTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisRangeTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 3, false);
		IrisDevUtil::AddInstanceMethod(this, "get_iterator", GetIterator, 0, false);
	}
	IrisRange();
	~IrisRange();
};

#endif