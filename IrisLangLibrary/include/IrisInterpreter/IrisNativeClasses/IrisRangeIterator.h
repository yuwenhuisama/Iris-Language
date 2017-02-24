#ifndef _H_IRISRANGEITERATOR_
#define _H_IRISRANGEITERATOR_

#include "IrisDevHeader.h"
#include "IrisRangeTag.h"
#include "IrisRangeIteratorTag.h"

class IrisRangeIterator : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue Next(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue IsEnd(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue GetValue(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "RangeIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisRangeIteratorTag);
	}

	void* NativeAlloc() {
		return new IrisRangeIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisRangeIteratorTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "next", Next, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}

	IrisRangeIterator();
	~IrisRangeIterator();
};

#endif