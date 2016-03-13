#ifndef _H_IRISCLASSBASE_
#define _H_IRISCLASSBASE_

#include "IrisDevHeader.h"

#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisClassBaseTag.h"
#include "IrisString.h"

class IrisClassBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

	static IrisValue GetClassName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		// 获取类指针
		IrisClassBaseTag* pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivObj);
		const string& strClassName = pClass->GetThisClassName();
		//创建String
		IIrisClass* pStringClass = IrisDevUtil::GetClass("String");
		return IrisDevUtil::CreateInstanceByInstantValue(strClassName.c_str());
	}

	static IrisValue New(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisClassBaseTag* pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivObj);
		return IrisDevUtil::CreateInstance(pClass->GetClass()->GetExternClass(), ivsVariableValues, pContextEnvironment);
	}

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
		IrisDevUtil::AddClassMethod(this, "new", New, 0, true);

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