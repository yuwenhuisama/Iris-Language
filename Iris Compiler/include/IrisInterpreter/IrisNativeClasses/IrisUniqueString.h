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
	static const IrisValue& GetUniqueString(size_t nIndex, bool& bResult) {
		unordered_map<size_t, IrisValue>::iterator iStr;
		if ((iStr = s_mpUniqueStringMap.find(nIndex)) != s_mpUniqueStringMap.end()) {
			bResult = true;
			return iStr->second;
		}
		bResult = false;
		return IrisDev_Nil();
	}

	static void AddUniqueString(size_t nIndex, const IrisValue& ivUniqueString) {
		s_mpUniqueStringMap[nIndex] = ivUniqueString;
	}

public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment) {
		return ivObj;
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "UniqueString";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDev_GetClass("Object");
	}

	static const string& GetString(IrisValue& ivValue) {
		return IrisDev_GetNativePointer<IrisUniqueStringTag*>(ivValue)->GetString();
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
		IrisDev_AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
	}

	//IrisValue CreateInstanceByUniqueIndex(unsigned int nIndex) {

	//	IrisValue ivValue;
	//	bool bResult = false;
	//	ivValue = GetUniqueString(nIndex, bResult);
	//	if (bResult) {
	//		return ivValue;
	//	}

	//	IrisObject* pObject = new IrisObject();
	//	pObject->SetClass(this);
	//	pObject->SetPermanent(true);
	//	IrisUniqueStringTag* pString = new IrisUniqueStringTag(IrisCompiler::CurrentCompiler()->GetUniqueString(nIndex, IrisCompiler::CurrentCompiler()->GetCurrentFileIndex()));
	//	pObject->SetNativeObject(pString);
	//	ivValue.SetIrisObject(pObject);

	//	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	//	IrisGC::CurrentGC()->Start();

	//	// 将新对象保存到堆里
	//	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);

	//	AddUniqueString(nIndex, ivValue);
	//	return ivValue;
	//}

	IrisUniqueString();
	~IrisUniqueString();
};

#endif