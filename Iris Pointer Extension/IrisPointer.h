#ifndef _H_IRISPOINTER_
#define _H_IRISPOINTER_

//* Always Needed
#include "..\IrisLangLibrary\include\IrisLangLibrary.h"
#pragma comment(lib, "IrisLangLibrary.lib")

//* User's Extention Class
#include "IrisPointerTag.h"

class IrisPointer : public IIrisClass
{
public:
	// Define native initialize method of this class
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		// Get native pointer
		auto pPointerTag = IrisDev::GetNativePointer<IrisPointerTag*>(ivObj);

		// Get parameter
		auto& ivLength = IrisDev::GetValue(ivsValues, 0);

		// Type check - well, it is supposed that you use type checking when you are developing a extension that will not only be uesed by yourself.
		if (!IrisDev::CheckClass(ivLength, "Integer")) {
			// If parameter get is no an integer, groan an Irregular.
			IrisDev::GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev::Nil();
		}

		// Get Integer Object's native int value
		int nLength = IrisDev::GetInt(ivLength);

		// Do initialize
		//IrisDev::addins(ivObj, "@length", IrisDev::CreateIntegerInstanceByInstantValue(nLength))
		IrisDev::SetObjectInstanceVariable(ivObj, "@length", IrisDev::CreateInstanceByInstantValue(nLength));
		pPointerTag->Initialize(nLength);

		return ivObj;
	}

	static IrisValue Get(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto& ivPoint = IrisDev::GetValue(ivsValues, 0);
		auto& ivLength = IrisDev::GetValue(ivsValues, 1);

		if (!IrisDev::CheckClass(ivPoint, "Integer")) {
			IrisDev::GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev::Nil();
		}

		if (!IrisDev::CheckClass(ivLength, "Integer")) {
			IrisDev::GroanIrregularWithString("Invaild parameter 2 : it must be a String or a Integer.");
			return IrisDev::Nil();
		}

		auto pPointer = IrisDev::GetNativePointer<IrisPointerTag*>(ivObj);
		int nPoint = IrisDev::GetInt(ivPoint);
		int nLength = IrisDev::GetInt(ivLength);
		string strResult;
		if (!pPointer->Get(nPoint, nLength, strResult)) {
			return IrisDev::False();
		}

		return IrisDev::CreateInstanceByInstantValue(strResult.c_str());
	}

	static IrisValue Set(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		auto& ivPoint = IrisDev::GetValue(ivsValues, 0);
		auto& ivDataString = IrisDev::GetValue(ivsValues, 1);

		if (!IrisDev::CheckClass(ivPoint, "Integer")) {
			IrisDev::GroanIrregularWithString("Invaild parameter 1 : it must be an Integer.");
			return IrisDev::Nil();
		}

		if (!IrisDev::CheckClass(ivDataString, "String") && !IrisDev::CheckClass(ivDataString, "UniqueString")) {
			IrisDev::GroanIrregularWithString("Invaild parameter 2 : it must be a String or a UniqueString.");
			return IrisDev::Nil();
		}

		auto pPointer = IrisDev::GetNativePointer<IrisPointerTag*>(ivObj);
		int nPoint = IrisDev::GetInt(ivPoint);
		const string& strData = IrisDev::GetString(ivDataString);

		return pPointer->Set(nPoint, strData.c_str(), strData.size()) ? IrisDev::True() : IrisDev::False();
	}

	static IrisValue GetLength(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return IrisDev::GetObjectInstanceVariable(ivObj, "@length");
	}

public:
	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Pointer";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev::GetClass("Object");
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
		IrisDev::AddInstanceMethod(this, "__format", InitializeFunction, 1, false);
		IrisDev::AddInstanceMethod(this, "get_data", Get, 2, false);
		IrisDev::AddInstanceMethod(this, "set_data", Set, 2, false);
		IrisDev::AddGetter(this, "@length", GetLength);
	}

	IrisPointer();
	~IrisPointer();
};

#endif