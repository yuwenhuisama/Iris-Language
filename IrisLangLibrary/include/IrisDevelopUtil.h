#ifndef _H_IRIDEVELOPUTIL_
#define _H_IRIDEVELOPUTIL_
#include "IrisUnil/IrisValue.h"
//#include "IrisThread/IrisThreadManager.h"
//#include <string>
//using namespace std;

class IIrisValues;
class IIrisClass;
class IIrisModule;
class IIrisInterface;
class IIrisContextEnvironment;
struct IrisThreadUniqueInfo;

typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

namespace IrisDevUtil {
	template<class T>
	T GetNativePointer(IrisValue& ivValue) {
		return static_cast<T>(ivValue.GetInstanceNativePointer());
	}

	template<class T>
	T GetNativePointer(IIrisObject* pObject) {
		return static_cast<T>(pObject->GetNativeObject());
	}

	IrisThreadUniqueInfo* GetCurrentThreadInfo();
	bool CurrentThreadIsMainThread();

	bool CheckClass(IrisValue& ivValue, const char* szClassName);

	void GroanIrregularWithString(const char* szIrregularString);

	int GetInt(IrisValue& ivValue);
	double GetFloat(IrisValue& ivValue);
	const char* GetString(IrisValue& ivValue);
	IrisValue CallMethod(IrisValue& ivObj, IIrisValues* pParameters, const char* szMethodName);
	IrisValue CreateInt(int nInteger);
	IrisValue CreateFloat(double dDouble);
	IrisValue CreateString(const char* nInteger);
	IIrisClass* GetClass(const char* strClassPathName);
	IIrisModule* GetModule(const char* strClassPathName);
	IIrisInterface* GetInterface(const char* strClassPathName);

	//IrisThreadUniqueInfo* GetCurrentThreadInfo();
	//
	//bool CurrentThreadIsMainThread();

	const IrisValue& Nil();
	const IrisValue& False();
	const IrisValue& True();

	void AddInstanceMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
	void AddClassMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

	void AddInstanceMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
	void AddClassMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

	void AddGetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);
	void AddSetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);

	void AddConstance(IIrisClass* pClass, const char* szConstanceName, const IrisValue& ivValue);
	void AddConstance(IIrisModule* pModule, const char* szConstanceName, const IrisValue& ivValue);

	void AddClassVariable(IIrisClass* pClass, const char* szVariableName, const IrisValue& ivValue);
	void AddClassVariable(IIrisModule* pClass, const char* szVariableName, const IrisValue& ivValue);

	void AddModule(IIrisClass* pClass, IIrisModule* pTargetModule);
	void AddModule(IIrisModule* pModule, IIrisModule* pTargetModule);

	IIrisClass* GetNativeClass(const IrisValue& ivValue);
	IIrisModule* GetNativeModule(const IrisValue& ivValue);
	IIrisInterface* GetNativeInterface(const IrisValue& ivValue);

	IrisValue CreateInstance(IIrisClass* pClass, IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment);
	IrisValue CreateInstanceByInstantValue(const char* szString);
	IrisValue CreateInstanceByInstantValue(double dFloat);
	IrisValue CreateInstanceByInstantValue(int nInteger);
	IrisValue CreateUniqueStringInstanceByUniqueIndex(size_t nIndex);

	bool IrregularHappened();
	bool FatalErrorHappened();
}

#endif