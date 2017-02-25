#ifndef _H_IRISCLASSBASE_
#define _H_IRISCLASSBASE_

#include "IrisDevHeader.h"

#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisClassBaseTag.h"
#include "IrisString.h"

class IrisClassBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue GetClassName(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);
	static IrisValue New(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo* pThreadInfo);

public:

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisClassBaseTag);
	}

	void Mark(void* pNativeObjectPointer) {}

	void* NativeAlloc() {
		return new IrisClassBaseTag(nullptr);
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisClassBaseTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);		
		IrisDevUtil::AddInstanceMethod(this, "new", New, 0, true);

		IrisDevUtil::AddGetter(this, "@class_name", GetClassName);
	}

	const char* NativeClassNameDefine() const {
		return "Class";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	IrisClassBase();
	~IrisClassBase();
};

#endif