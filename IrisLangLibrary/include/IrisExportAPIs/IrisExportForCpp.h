/*
 * Development APIs exported using C++ ABI.
 * C++ API will provide APIs of style :
 * IrisDevUtil::<FunctionName>(Object, parameter);
*/

#ifndef _H_IRISEXPROTFORCPP_
#define _H_IRISEXPROTFORCPP_

#ifdef IRISLANGLIBRARY_EXPORTS
#define IRISLANGLIBRARY_API __declspec(dllexport)
#else
#define IRISLANGLIBRARY_API __declspec(dllimport)
#endif

#include "IrisUnil/IrisValue.h"
#include "IrisInterfaces/IIrisClass.h"
#include "IrisInterfaces/IIrisClosureBlock.h"
#include "IrisInterfaces/IIrisContextEnvironment.h"
#include "IrisInterfaces/IIrisInterface.h"
#include "IrisInterfaces/IIrisModule.h"
#include "IrisInterfaces/IIrisObject.h"
#include "IrisInterfaces/IIrisValues.h"

namespace IrisDev {
	typedef IrisValue(*IrisNativeFunction)(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

#define DECLARE_IRISDEV_CLASS_CHECK(klass) IRISLANGLIBRARY_API bool CheckClassIs##klass(const IrisValue& ivValue);

	DECLARE_IRISDEV_CLASS_CHECK(Class)
	DECLARE_IRISDEV_CLASS_CHECK(Module)
	DECLARE_IRISDEV_CLASS_CHECK(Interface)
	DECLARE_IRISDEV_CLASS_CHECK(Object)
	DECLARE_IRISDEV_CLASS_CHECK(String)
	DECLARE_IRISDEV_CLASS_CHECK(UniqueString)
	DECLARE_IRISDEV_CLASS_CHECK(Integer)
	DECLARE_IRISDEV_CLASS_CHECK(Float)
	DECLARE_IRISDEV_CLASS_CHECK(Array)
	DECLARE_IRISDEV_CLASS_CHECK(Hash)
	DECLARE_IRISDEV_CLASS_CHECK(Range)
	DECLARE_IRISDEV_CLASS_CHECK(Block)

	IRISLANGLIBRARY_API void* _InnerGetNativePointer(const IrisValue& ivValue);
	IRISLANGLIBRARY_API void* _InnerGetNativePointer(IIrisObject* pObject);

#ifndef _IRIS_LIB_DEFINE_
#define _IRIS_LIB_DEFINE_
	template<class T>
	T GetNativePointer(const IrisValue& ivValue) {
		return static_cast<T>(_InnerGetNativePointer(ivValue));
	}

	template<class T>
	T GetNativePointer(IIrisObject* pObject) {
		return static_cast<T>(_InnerGetNativePointer(pObject));
	}
#endif

	IRISLANGLIBRARY_API bool CheckClass(const IrisValue& ivValue, const char* szClassName);

	IRISLANGLIBRARY_API bool CheckClassIsStringOrUniqueString(const IrisValue& ivValue);

	IRISLANGLIBRARY_API void GroanIrregularWithString(const char* szIrregularString);

	IRISLANGLIBRARY_API int				GetInt(const IrisValue& ivValue);
	IRISLANGLIBRARY_API double			GetFloat(const IrisValue& ivValue);
	IRISLANGLIBRARY_API const char*		GetString(const IrisValue& ivValue);
	IRISLANGLIBRARY_API IrisValue		CallMethod(const IrisValue& ivObj, const char* szMethodName, IIrisValues* pParameters);
	IRISLANGLIBRARY_API IrisValue		CallClassClassMethod(IIrisClass* pClass, const char* szMethodName, IIrisValues* pParameters);
	IRISLANGLIBRARY_API IrisValue		CallClassModuleMethod(IIrisModule* pModule, const char* szMethodName, IIrisValues* pParameters);
	IRISLANGLIBRARY_API IIrisClass*		GetClass(const char* strClassPathName);
	IRISLANGLIBRARY_API IIrisModule*	GetModule(const char* strClassPathName);
	IRISLANGLIBRARY_API IIrisInterface* GetInterface(const char* strClassPathName);

	IRISLANGLIBRARY_API bool				ObjectIsFixed(const IrisValue& ivObj);
	IRISLANGLIBRARY_API IIrisClosureBlock*	GetClosureBlock(IIrisContextEnvironment* pContextEnvironment);
	IRISLANGLIBRARY_API IrisValue			ExcuteClosureBlock(IIrisClosureBlock* pClosureBlock, IIrisValues* pParameters);
	IRISLANGLIBRARY_API void				ContextEnvironmentSetClosureBlock(IIrisContextEnvironment* pContextEnvironment, IIrisClosureBlock* pBlock);
	IRISLANGLIBRARY_API IIrisObject*		GetNativeObjectPointer(const IrisValue& ivObj);
	IRISLANGLIBRARY_API int					GetObjectID(const IrisValue& ivObj);
	IRISLANGLIBRARY_API IIrisClass*			GetClassOfObject(const IrisValue& ivObj);
	IRISLANGLIBRARY_API const char*			GetNameOfClass(IIrisClass* pClass);
	IRISLANGLIBRARY_API const char*			GetNameOfModule(IIrisModule* pModule);
	IRISLANGLIBRARY_API const char*			GetNameOfInterface(IIrisInterface* pInterface);
	IRISLANGLIBRARY_API void				MarkObject(const IrisValue& ivObject);
	IRISLANGLIBRARY_API void				MarkClosureBlock(IIrisClosureBlock* pClosureBlock);

	IRISLANGLIBRARY_API const IrisValue& Nil();
	IRISLANGLIBRARY_API const IrisValue& False();
	IRISLANGLIBRARY_API const IrisValue& True();

	IRISLANGLIBRARY_API bool RegistClass(const char* szPath, IIrisClass* pClass);
	IRISLANGLIBRARY_API bool RegistModule(const char* szPath, IIrisModule* pModule);
	IRISLANGLIBRARY_API bool RegistInterface(const char* szPath, IIrisInterface* pInterface);

	//IRISLANGLIBRARY_API bool AddClass(IIrisModule* pModule, IIrisClass* pClass);

	IRISLANGLIBRARY_API void AddInstanceMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
	IRISLANGLIBRARY_API void AddClassMethod(IIrisClass* pClass, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

	IRISLANGLIBRARY_API void AddInstanceMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);
	IRISLANGLIBRARY_API void AddClassMethod(IIrisModule* pModule, const char* szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter);

	IRISLANGLIBRARY_API void AddGetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);
	IRISLANGLIBRARY_API void AddSetter(IIrisClass* pClass, const char* szInstanceVariableName, IrisNativeFunction pfFunction);

	IRISLANGLIBRARY_API void AddConstance(IIrisClass* pClass, const char* szConstanceName, const IrisValue& ivValue);
	IRISLANGLIBRARY_API void AddConstance(IIrisModule* pModule, const char* szConstanceName, const IrisValue& ivValue);

	IRISLANGLIBRARY_API void AddClassVariable(IIrisClass* pClass, const char* szVariableName, const IrisValue& ivValue);
	IRISLANGLIBRARY_API void AddClassVariable(IIrisModule* pModule, const char* szVariableName, const IrisValue& ivValue);

	IRISLANGLIBRARY_API void AddModule(IIrisClass* pClass, IIrisModule* pTargetModule);
	IRISLANGLIBRARY_API void AddModule(IIrisModule* pModule, IIrisModule* pTargetModule);

	IRISLANGLIBRARY_API IrisValue CreateNormalInstance(IIrisClass* pClass, IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment);
	IRISLANGLIBRARY_API IrisValue CreateInstanceByInstantValue(const char* szString);
	IRISLANGLIBRARY_API IrisValue CreateInstanceByInstantValue(double dFloat);
	IRISLANGLIBRARY_API IrisValue CreateInstanceByInstantValue(int nInteger);
	IRISLANGLIBRARY_API IrisValue CreateUniqueStringInstanceByUniqueIndex(size_t nIndex);

	IRISLANGLIBRARY_API void	  SetObjectInstanceVariable(const IrisValue& ivObj, char* szInstanceVariableName, const IrisValue& ivValue);
	IRISLANGLIBRARY_API IrisValue GetObjectInstanceVariable(const IrisValue& ivObj, char* szInstanceVariableName);

	IRISLANGLIBRARY_API bool IrregularHappened();
	IRISLANGLIBRARY_API bool FatalErrorHappened();

	IRISLANGLIBRARY_API IIrisValues*		CreateIrisValuesList(size_t nSize);
	IRISLANGLIBRARY_API const IrisValue&	GetValue(IIrisValues* pValues, size_t nIndex);
	IRISLANGLIBRARY_API void				SetValue(IIrisValues* pValues, size_t nIndex, const IrisValue& ivValue);
	IRISLANGLIBRARY_API void		    	ReleaseIrisValuesList(IIrisValues* pValues);
};

#ifndef  _IRIS_INITIALIZE_STRUCT_DEFINED_
#define _IRIS_INITIALIZE_STRUCT_DEFINED_

typedef void(*FatalErrorMessageFunctionForCpp)(const char* pMessage);
typedef bool(*ExitConditionFunctionForCpp)();

typedef struct IrisInitializeStructForCpp_tag {
	FatalErrorMessageFunctionForCpp m_pfFatalErrorMessageFunction;
	ExitConditionFunctionForCpp m_pfExitConditionFunction;
} IrisInitializeStructForCpp, *PIrisInitializeStructForCpp;

#endif // ! _IRIS_INITIALIZE_STRUCT_DEFINED_


IRISLANGLIBRARY_API bool IR_Initialize(PIrisInitializeStructForCpp pInitializeStruct);
IRISLANGLIBRARY_API bool IR_Run();
IRISLANGLIBRARY_API bool IR_ShutDown();

IRISLANGLIBRARY_API bool IR_LoadScriptFromPath(const char* pScriptFilePath);
IRISLANGLIBRARY_API bool IR_LoadScriptFromVirtualPathAndText(const char* pPath, const char * pScriptText);

IRISLANGLIBRARY_API bool IR_LoadExtention(const char* pExtentionPath);

#endif