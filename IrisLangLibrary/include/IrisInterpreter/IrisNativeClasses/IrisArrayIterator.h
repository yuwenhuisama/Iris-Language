#ifndef _H_IRISARRAYITERATOR_
#define _H_IRISARRAYITERATOR_

#include "IrisDevHeader.h"
#include "IrisArrayTag.h"
#include "IrisArrayIteratorTag.h"

class IrisArrayIterator : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue Next(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue IsEnd(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);
	static IrisValue GetValue(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:
	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisArrayIteratorTag);
	}

	void Mark(void* pNativeObjectPointer) { }

	void* NativeAlloc() {
		return new IrisArrayIteratorTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisArrayIteratorTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "next", Next, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "is_end", IsEnd, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "get_value", GetValue, 0, false);
	}
	
	const char* NativeClassNameDefine() const {
		return "ArrayIterator";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	IrisArrayIterator();
	~IrisArrayIterator();
};

#endif