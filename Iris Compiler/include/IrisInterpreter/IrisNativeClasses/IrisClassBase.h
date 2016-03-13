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
		IrisClassBaseTag* pClass = IrisDev_GetNativePointer<IrisClassBaseTag*>(ivObj);
		const string& strClassName = pClass->GetThisClassName();
		//创建String
		IIrisClass* pStringClass = IrisDev_GetClass("String");
		return IrisDev_CreateInstanceByInstantValue(strClassName.c_str());
	}

	static IrisValue New(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		IrisClassBaseTag* pClass = IrisDev_GetNativePointer<IrisClassBaseTag*>(ivObj);
		return IrisDev_CreateInstance(pClass->GetClass()->GetExternClass(), ivsVariableValues, pContextEnvironment);
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
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);		
		IrisDev_AddClassMethod(this, "new", New, 0, true);

		IrisDev_AddGetter(this, "@class_name", GetClassName);
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