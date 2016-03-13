#ifndef _H_IRISPOINTER_
#define _H_IRISPOINTER_

//* Always Needed
#include "IrisLangLibrary.h"
#pragma comment(lib, "IrisLangLibrary.lib")

//* User's Extention Class
#include "IrisPointerTag.h"

class IrisPointer : public IIrisClass
{
public:
	// Define native initialize method of this class
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		// Get native pointer
		auto pPointerTag = IrisDev_GetNativePointer<IrisPointerTag*>(ivObj);

		// Get parameter
		auto& ivLength = ivsValues->GetValue(0);

		// Type check - well, it is supposed that you use type checking when you are developing a extension that will not only be uesed by yourself.
		if (!IrisDev_CheckClass(ivLength, "Integer")) {
			// If parameter get is no an integer, groan an Irregular.
			IrisDev_GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev_Nil();
		}

		// Get Integer Object's native int value
		int nLength = IrisDev_GetInt(ivLength);

		// Do initialize
		ivObj.GetIrisObject()->AddInstanceValue("@length", IrisDev_CreateIntegerInstanceByInstantValue(nLength));
		pPointerTag->Initialize(nLength);

		return ivObj;
	}

	static IrisValue Get(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto& ivPoint = ivsValues->GetValue(0);
		auto& ivLength = ivsValues->GetValue(1);

		if (!IrisDev_CheckClass(ivPoint, "Integer")) {
			IrisDev_GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev_Nil();
		}

		if (!IrisDev_CheckClass(ivLength, "Integer")) {
			IrisDev_GroanIrregularWithString("Invaild parameter 2 : it must be a String or a Integer.");
			return IrisDev_Nil();
		}

		auto pPointer = IrisDev_GetNativePointer<IrisPointerTag*>(ivObj);
		int nPoint = IrisDev_GetInt(ivPoint);
		int nLength = IrisDev_GetInt(ivLength);
		string strResult;
		if (!pPointer->Get(nPoint, nLength, strResult)) {
			return IrisDev_False();
		}

		return IrisDev_CreateStringInstanceByInstantValue(strResult.c_str());
	}

	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto& ivPoint = ivsValues->GetValue(0);
		auto& ivDataString = ivsValues->GetValue(1);

		if (!IrisDev_CheckClass(ivPoint, "Integer")) {
			IrisDev_GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev_Nil();
		}

		if (!IrisDev_CheckClass(ivDataString, "String") && !IrisDev_CheckClass(ivDataString, "UniqueString")) {
			IrisDev_GroanIrregularWithString("Invaild parameter 2 : it must be a String or a UniqueString.");
			return IrisDev_Nil();
		}

		auto pPointer = IrisDev_GetNativePointer<IrisPointerTag*>(ivObj);
		int nPoint = IrisDev_GetInt(ivPoint);
		const string& strData = IrisDev_GetString(ivDataString);

		return pPointer->Set(nPoint, strData.c_str(), strData.size()) ? IrisDev_True() : IrisDev_False();
	}

	static IrisValue GetLength(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		bool bResult = false;
		return ivObj.GetIrisObject()->GetInstanceValue("@length", bResult);
	}

public:
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Pointer";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	// Overwrite virtual method GetTrustteeSize: To tell Iris how much memory has been malloced (GC will use this information).
	int GetTrustteeSize(void* pNativePointer) {
		return static_cast<IrisPointerTag*>(pNativePointer)->GetTrustteeSize();
	}

	// Overwrite virtual method NativeAlloc : To give Iris a native object, and Iris will create an Object to link to it.
	void* NativeAlloc() {
		return new IrisPointerTag();
	}

	// Overwrite virtual method NativeFree : When Iris's GC release a object of this class, and native object linked to it should also be released.
	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisPointerTag*>(pNativePointer);
	}

	// Overwrite virtual method NativeClassDefine : To add instance method/class method/getter/setter and others into this class.
	void NativeClassDefine() {
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDev_AddInstanceMethod(this, "get_data", Get, 2, false);
		IrisDev_AddInstanceMethod(this, "set_data", Set, 2, false);
		IrisDev_AddGetter(this, "@length", GetLength);
	}

	IrisPointer();
	~IrisPointer();
};

#endif