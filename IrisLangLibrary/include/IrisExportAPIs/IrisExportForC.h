#ifndef _H_IRISEXPORTFORC_
#define _H_IRISEXPORTFORC_

#ifdef IRISLANGLIBRARY_EXPORTS
#define IRISLANGLIBRARY_API __declspec(dllexport)
#else
#define IRISLANGLIBRARY_API __declspec(dllimport)
#endif

extern "C" {
	typedef struct CIrisValue_tag {
		void* m_pObject;
	} CIrisValue;

	typedef void* CIrisNativeFunction;
	typedef void* CIIrisObject;
	typedef void* CIIrisValues;
	typedef void* CIIrisClass;
	typedef void* CIIrisModule;
	typedef void* CIIrisInterface;
	typedef void* CIIrisClosureBlock;
	typedef void* CIIrisContextEnvironment;
	typedef void* CIrisNativeFunction;
	typedef void* CIIrisThreadInfo;

#define DECLARE_IRISDEV_CLASS_CHECK(klass) IRISLANGLIBRARY_API bool CheckClassIs##klass(CIrisValue ivValue);

#define IrisDev_GetNativePointerWithValue(type, value) (type)_IrisDev_InnerGetNativePointerWithValue(value)
#define IrisDev_GetNativePointerWithObject(type, object) (type)_IrisDev_InnerGetNativePointerWithObject(object)

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

	IRISLANGLIBRARY_API void* _IrisDev_InnerGetNativePointerWithValue(CIrisValue ivValue);
	IRISLANGLIBRARY_API void* _IrisDev_InnerGetNativePointerWithObject(CIIrisObject pObject);

	IRISLANGLIBRARY_API int IrisDev_CheckClass(CIrisValue ivValue, char* szClassName);
	IRISLANGLIBRARY_API void IrisDev_GroanIrregularWithString(char* szIrregularString, CIIrisThreadInfo pThreadInfo);

	IRISLANGLIBRARY_API int					IrisDev_GetInt(CIrisValue ivValue);
	IRISLANGLIBRARY_API double				IrisDev_GetFloat(CIrisValue ivValue);
	IRISLANGLIBRARY_API char*				IrisDev_GetString(CIrisValue ivValue);
	IRISLANGLIBRARY_API CIrisValue			IrisDev_CallMethod(CIrisValue ivObj, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API CIrisValue			IrisDev_CallClassClassMethod(CIIrisClass pClass, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API CIrisValue			IrisDev_CallClassModuleMethod(CIIrisModule pModule, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API CIIrisClass			IrisDev_GetClass(char* strClassPathName);
	IRISLANGLIBRARY_API CIIrisModule		IrisDev_GetModule(char* strClassPathName);
	IRISLANGLIBRARY_API CIIrisInterface		IrisDev_GetInterface(char* strClassPathName);

	IRISLANGLIBRARY_API int					IrisDev_ObjectIsFixed(CIrisValue ivObj);
	IRISLANGLIBRARY_API CIIrisClosureBlock	IrisDev_GetClosureBlock(CIIrisContextEnvironment pContextEnvironment);
	IRISLANGLIBRARY_API CIrisValue			IrisDev_ExcuteClosureBlock(CIIrisClosureBlock pClosureBlock, CIIrisValues pParameters, CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API void				IrisDev_ContextEnvironmentSetClosureBlock(CIIrisContextEnvironment pContextEnvironment, CIIrisClosureBlock pBlock);
	IRISLANGLIBRARY_API CIIrisObject		IrisDev_GetNativeObjectPointer(CIrisValue ivObj);
	IRISLANGLIBRARY_API int					IrisDev_GetObjectID(CIrisValue ivObj);
	IRISLANGLIBRARY_API CIIrisClass			IrisDev_GetClassOfObject(CIrisValue ivObj);
	IRISLANGLIBRARY_API char*				IrisDev_GetNameOfClass(CIIrisClass pClass);
	IRISLANGLIBRARY_API char*				IrisDev_GetNameOfModule(CIIrisModule pModule);
	IRISLANGLIBRARY_API char*				IrisDev_GetNameOfInterface(CIIrisInterface pInterface);
	IRISLANGLIBRARY_API void				IrisDev_MarkObject(CIrisValue ivObject);
	IRISLANGLIBRARY_API void				IrisDev_MarkClosureBlock(CIIrisClosureBlock pClosureBlock);

	IRISLANGLIBRARY_API CIrisValue IrisDev_Nil();
	IRISLANGLIBRARY_API CIrisValue IrisDev_False();
	IRISLANGLIBRARY_API CIrisValue IrisDev_True();

	IRISLANGLIBRARY_API int IrisDev_RegistClass(char* szPath, CIIrisClass pClass);
	IRISLANGLIBRARY_API int IrisDev_RegistModule(char* szPath, CIIrisModule pModule);
	IRISLANGLIBRARY_API int IrisDev_RegistInterface(char* szPath, CIIrisInterface pInterface);

	IRISLANGLIBRARY_API void IrisDev_ClassAddInstanceMethod(CIIrisClass pClass, char* szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter);
	IRISLANGLIBRARY_API void IrisDev_ClassAddClassMethod(CIIrisClass pClass, char* szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter);

	IRISLANGLIBRARY_API void IrisDev_ModuleAddInstanceMethod(CIIrisModule pModule, char* szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter);
	IRISLANGLIBRARY_API void IrisDev_ModuleAddClassMethod(CIIrisModule pModule, char* szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter);

	IRISLANGLIBRARY_API void IrisDev_AddGetter(CIIrisClass pClass, char* szInstanceVariableName, CIrisNativeFunction pfFunction);
	IRISLANGLIBRARY_API void IrisDev_AddSetter(CIIrisClass pClass, char* szInstanceVariableName, CIrisNativeFunction pfFunction);

	IRISLANGLIBRARY_API void IrisDev_ClassAddConstance(CIIrisClass pClass, char* szConstanceName, CIrisValue ivValue);
	IRISLANGLIBRARY_API void IrisDev_ModuleAddConstance(CIIrisModule pModule, char* szConstanceName, CIrisValue ivValue);

	IRISLANGLIBRARY_API void IrisDev_ClassAddClassVariable(CIIrisClass pClass, char* szVariableName, CIrisValue ivValue);
	IRISLANGLIBRARY_API void IrisDev_ModuleAddClassVariable(CIIrisModule pModule, char* szVariableName, CIrisValue ivValue);

	IRISLANGLIBRARY_API void IrisDev_ClassAddModule(CIIrisClass pClass, CIIrisModule pTargetModule);
	IRISLANGLIBRARY_API void IrisDev_ModuleAddModule(CIIrisModule pModule, CIIrisModule pTargetModule);

	IRISLANGLIBRARY_API CIrisValue IrisDev_CreateNormalInstance(CIIrisClass pClass, CIIrisValues ivsParams, CIIrisContextEnvironment pContexEnvironment, CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API CIrisValue IrisDev_CreateStringInstanceByInstantValue(char* szString);
	IRISLANGLIBRARY_API CIrisValue IrisDev_CreateFloatInstanceByInstantValue(double dFloat);
	IRISLANGLIBRARY_API CIrisValue IrisDev_CreateIntegerInstanceByInstantValue(int nInteger);
	IRISLANGLIBRARY_API CIrisValue IrisDev_CreateUniqueStringInstanceByUniqueIndex(size_t nIndex);

	IRISLANGLIBRARY_API void	   IrisDev_SetObjectInstanceVariable(CIrisValue& ivObj, char* szInstanceVariableName, CIrisValue ivValue);
	IRISLANGLIBRARY_API CIrisValue IrisDev_GetObjectInstanceVariable(CIrisValue& ivObj, char* szInstanceVariableName);

	IRISLANGLIBRARY_API int IrisDev_IrregularHappened(CIIrisThreadInfo pThreadInfo);
	IRISLANGLIBRARY_API int IrisDev_FatalErrorHappened(CIIrisThreadInfo pThreadInfo);

	IRISLANGLIBRARY_API CIIrisValues		IrisDev_CreateIrisValuesList(size_t nSize);
	IRISLANGLIBRARY_API CIrisValue			IrisDev_GetValue(CIIrisValues pValues, size_t nIndex);
	IRISLANGLIBRARY_API void				IrisDev_SetValue(CIIrisValues pValues, size_t nIndex, CIrisValue ivValue);
	IRISLANGLIBRARY_API void		    	IrisDev_ReleaseIrisValuesList(CIIrisValues pValues);

#ifndef  _IRIS_INITIALIZE_STRUCT_DEFINED_
#define _IRIS_INITIALIZE_STRUCT_DEFINED_
	typedef void(*FatalErrorMessageFunctionForC)(char* pMessage);
	typedef int(*ExitConditionFunctionForC)();

	typedef struct IrisInitializeStructForC_tag {
		FatalErrorMessageFunctionForC m_pfFatalErrorMessageFunction;
		ExitConditionFunctionForC m_pfExitConditionFunction;
	} IrisInitializeStructForC, *PIrisInitializeStructForC;

#endif // ! _IRIS_INITIALIZE_STRUCT_DEFINED_

	IRISLANGLIBRARY_API int IR_Initialize(PIrisInitializeStructForC pInitializeStruct);
	IRISLANGLIBRARY_API int IR_Run();
	IRISLANGLIBRARY_API int IR_ShutDown();

	IRISLANGLIBRARY_API int IR_LoadScriptFromPath(char* pScriptFilePath);
	IRISLANGLIBRARY_API int IR_LoadScriptFromVirtualPathAndText(char* pPath, char* pScriptText);

	IRISLANGLIBRARY_API int IR_LoadExtention(char* pExtentionPath);

}

#endif