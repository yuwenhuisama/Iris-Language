#ifndef _H_IRISUNIQUESTRING_
#define _H_IRISUNIQUESTRING_

#include "IrisDevHeader.h"
#include "IrisIntegerTag.h"
#include "IrisUniqueStringTag.h"
#include "IrisClassBase.h"
#include "IrisCompiler.h"

#include <string.h>
#include <unordered_map>
using namespace std;

class IrisUniqueString : public IIrisClass
{
private:
	static unordered_map<size_t, IrisValue> s_mpUniqueStringMap;

public:
	static const IrisValue& GetUniqueString(size_t nIndex, bool& bResult);
	static void AddUniqueString(size_t nIndex, const IrisValue& ivUniqueString);

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "UniqueString";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	static const string& GetString(const IrisValue& ivValue) {
		return IrisDevUtil::GetNativePointer<IrisUniqueStringTag*>(ivValue)->GetString();
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisUniqueStringTag);
	}

	void* NativeAlloc() {
		return new IrisUniqueStringTag("");
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisUniqueStringTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "to_string", ToString, 0, false);
	}

	IrisUniqueString();
	~IrisUniqueString();
};

#endif