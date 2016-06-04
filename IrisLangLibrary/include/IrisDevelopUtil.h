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
class IIrisClosureBlock;
struct IrisThreadUniqueInfo;

typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

#define DECLARE_CLASS_CHECK(klass) bool CheckClassIs##klass(const IrisValue& ivValue);
#define IMPLEMENT_CLASS_CHECK(klass) bool CheckClassIs##klass(const IrisValue& ivValue) { return INNER_CLASS_GET_POINTER(klass) == static_cast<IrisObject*>(ivValue.GetIrisObject())->GetClass();  }

namespace IrisDevUtil {
	template<class T>
	T GetNativePointer(const IrisValue& ivValue) {
		//return static_cast<T>(ivValue.GetInstanceNativePointer());
		return static_cast<T>(_InnerGetNativePointer(ivValue));
	}

	template<class T>
	T GetNativePointer(IIrisObject* pObject) {
		//return static_cast<T>(pObject->GetNativeObject());
		return static_cast<T>(_InnerGetNativePointer(pObject));
	}

	IrisThreadUniqueInfo* GetCurrentThreadInfo();
	bool CurrentThreadIsMainThread();

	bool CheckClass(const IrisValue& ivValue, const char* szClassName);

	DECLARE_CLASS_CHECK(Class)
	DECLARE_CLASS_CHECK(Module)
	DECLARE_CLASS_CHECK(Interface)
	DECLARE_CLASS_CHECK(Object)
	DECLARE_CLASS_CHECK(String)
	DECLARE_CLASS_CHECK(UniqueString)
	DECLARE_CLASS_CHECK(Integer)
	DECLARE_CLASS_CHECK(Float)
	DECLARE_CLASS_CHECK(Array)
	DECLARE_CLASS_CHECK(Hash)
	DECLARE_CLASS_CHECK(Range)
	DECLARE_CLASS_CHECK(Block)

	void GroanIrregularWithString(const char* szIrregularString);

	int GetInt(const IrisValue& ivValue);
	double GetFloat(const IrisValue& ivValue);
	const char* GetString(const IrisValue& ivValue);
	IrisValue CallMethod(const IrisValue& ivObj, IIrisValues* pParameters, const char* szMethodName);
	IrisValue CreateInt(int nInteger);
	IrisValue CreateFloat(double dDouble);
	IrisValue CreateString(const char* nInteger);
	IIrisClass* GetClass(const char* strClassPathName);
	IIrisModule* GetModule(const char* strClassPathName);
	IIrisInterface* GetInterface(const char* strClassPathName);

	bool ObjectIsFixed(const IrisValue& ivObj);
	IIrisClosureBlock* GetClosureBlock(IIrisContextEnvironment* pContextEnvironment);
	IrisValue ExcuteClosureBlock(IIrisClosureBlock* pClosureBlock, IIrisValues* pParameters);
	void ContextEnvironmentSetClosureBlock(IIrisContextEnvironment* pContextEnvironment, IIrisClosureBlock* pBlock);
	IIrisObject* GetNativeObjectPointer(const IrisValue& ivObj);
	int GetObjectID(const IrisValue& ivObj);
	IIrisClass* GetClassOfObject(const IrisValue& ivObj);
	const char* GetNameOfClass(IIrisClass* pClass);
	const char* GetNameOfModule(IIrisModule* pModule);
	const char* GetNameOfInterface(IIrisInterface* pInterface);

	void SetObjectInstanceVariable(IrisValue& ivObj, char* szInstanceVariableName, const IrisValue& ivValue);
	IrisValue GetObjectInstanceVariable(IrisValue& ivObj, char* szInstanceVariableName);

	void MarkObject(const IrisValue& ivObject);
	void MarkClosureBlock(IIrisClosureBlock* pClosureBlock);

	//IrisThreadUniqueInfo* GetCurrentThreadInfo();
	//
	//bool CurrentThreadIsMainThread();

	void* _InnerGetNativePointer(const IrisValue& ivValue);
	void* _InnerGetNativePointer(IIrisObject* pObject);

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