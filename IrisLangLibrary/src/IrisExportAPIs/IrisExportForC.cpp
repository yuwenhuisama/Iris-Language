#include "IrisExportAPIs\IrisExportForC.h"

#include "stdafx.h"

#include "IrisDevelopUtil.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInteger.h"
#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"
#include "IrisInterpreter/IrisNativeClasses/IrisString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.H"
#include "IrisFatalErrorHandler.h"

FatalErrorMessageFunctionForC g_pfFatalErrorMessageFunctionForC;
ExitConditionFunctionForC g_pfExitConditionFunctionForC;

#define IMPLEMENT_IRISDEV_CLASS_CHECK(klass) IRISLANGLIBRARY_API bool IrisDev_CheckClassIs##klass(CIrisValue ivValue) { return IrisDevUtil::CheckClassIs##klass(*((IrisValue*)&(ivValue)));  }

IMPLEMENT_IRISDEV_CLASS_CHECK(Class)
IMPLEMENT_IRISDEV_CLASS_CHECK(Module)
IMPLEMENT_IRISDEV_CLASS_CHECK(Interface)
IMPLEMENT_IRISDEV_CLASS_CHECK(Object)
IMPLEMENT_IRISDEV_CLASS_CHECK(String)
IMPLEMENT_IRISDEV_CLASS_CHECK(UniqueString)
IMPLEMENT_IRISDEV_CLASS_CHECK(Integer)
IMPLEMENT_IRISDEV_CLASS_CHECK(Float)
IMPLEMENT_IRISDEV_CLASS_CHECK(Array)
IMPLEMENT_IRISDEV_CLASS_CHECK(Hash)
IMPLEMENT_IRISDEV_CLASS_CHECK(Range)
IMPLEMENT_IRISDEV_CLASS_CHECK(Block)

#define CAST_AS_IrisValue(value)				(*((IrisValue*)&(value)))
#define CAST_AS_CIrisValue(value)				(*((CIrisValue*)&(value)))
#define CAST_AS_IIrisObject(value)				((IIrisObject*)(value))
#define CAST_AS_IIrisValues(value)				((IIrisValues*)(value))
#define CAST_AS_IIrisClass(value)				((IIrisClass*)(value))
#define CAST_AS_IIrisModule(value)				((IIrisModule*)(value))
#define CAST_AS_IIrisInterface(value)			((IIrisInterface*)(value))
#define CAST_AS_IIrisContextEnvironment(value)  ((IIrisContextEnvironment*)(value))
#define CAST_AS_IIrisClosureBlock(value)		((IIrisClosureBlock*)(value))
#define CAST_AS_IIrisThreadInfo(value)			((IIrisThreadInfo*)(value))

IRISLANGLIBRARY_API void* _IrisDev_InnerGetNativePointerWithValue(CIrisValue ivValue) {
	return IrisDevUtil::_InnerGetNativePointer(CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void* _IrisDev_InnerGetNativePointerWithObject(CIIrisObject pObject) {
	return IrisDevUtil::_InnerGetNativePointer(CAST_AS_IIrisObject(pObject));
}

IRISLANGLIBRARY_API int IrisDev_CheckClass(CIrisValue ivValue, char * szClassName)
{
	return IrisDevUtil::CheckClass(CAST_AS_IrisValue(ivValue), szClassName) ? 1 : 0;
}

IRISLANGLIBRARY_API void IrisDev_GroanIrregularWithString(char * szIrregularString, CIIrisThreadInfo pThreadInfo)
{
	IrisDevUtil::GroanIrregularWithString(szIrregularString, CAST_AS_IIrisThreadInfo(pThreadInfo));
}

IRISLANGLIBRARY_API int IrisDev_GetInt(CIrisValue ivValue)
{
	return IrisDevUtil::GetInt(CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API double IrisDev_GetFloat(CIrisValue ivValue)
{
	return IrisDevUtil::GetFloat(CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API char * IrisDev_GetString(CIrisValue ivValue)
{
	return (char*)(IrisDevUtil::GetString(CAST_AS_IrisValue(ivValue)));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CallMethod(CIrisValue ivObj, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo)
{
	return CAST_AS_CIrisValue((IrisDevUtil::CallMethod(CAST_AS_IrisValue(ivObj), szMethodName, CAST_AS_IIrisValues(pParameters), CAST_AS_IIrisContextEnvironment(pContextEnvironment), CAST_AS_IIrisThreadInfo(pThreadInfo))));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CallClassClassMethod(CIIrisClass pClass, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo)
{
	return CAST_AS_CIrisValue((IrisDevUtil::CallClassMethod(CAST_AS_IIrisClass(pClass), szMethodName, CAST_AS_IIrisValues(pParameters), CAST_AS_IIrisContextEnvironment(pContextEnvironment), CAST_AS_IIrisThreadInfo(pThreadInfo))));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CallClassModuleMethod(CIIrisModule pModule, char* szMethodName, CIIrisValues pParameters, CIIrisContextEnvironment pContextEnvironment, CIIrisThreadInfo pThreadInfo)
{
	return CAST_AS_CIrisValue((IrisDevUtil::CallClassMethod(CAST_AS_IIrisModule(pModule), szMethodName, CAST_AS_IIrisValues(pParameters), CAST_AS_IIrisContextEnvironment(pContextEnvironment), CAST_AS_IIrisThreadInfo(pThreadInfo))));
}

IRISLANGLIBRARY_API CIIrisClass IrisDev_GetClass(char * strClassPathName)
{
	return IrisDevUtil::GetClass(strClassPathName);
}

IRISLANGLIBRARY_API CIIrisModule IrisDev_GetModule(char * strClassPathName)
{
	return IrisDevUtil::GetModule(strClassPathName);
}

IRISLANGLIBRARY_API CIIrisInterface IrisDev_GetInterface(char * strClassPathName)
{
	return IrisDevUtil::GetInterface(strClassPathName);
}

IRISLANGLIBRARY_API int IrisDev_ObjectIsFixed(CIrisValue ivObj)
{
	return IrisDevUtil::ObjectIsFixed(CAST_AS_IrisValue(ivObj)) ? 1 : 0;
}

IRISLANGLIBRARY_API CIIrisClosureBlock IrisDev_GetClosureBlock(CIIrisContextEnvironment pContextEnvironment)
{
	return IrisDevUtil::GetClosureBlock(CAST_AS_IIrisContextEnvironment(pContextEnvironment));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_ExcuteClosureBlock(CIIrisClosureBlock pClosureBlock, CIIrisValues pParameters, CIIrisThreadInfo pThreadInfo)
{
	return CAST_AS_CIrisValue(IrisDevUtil::ExcuteClosureBlock(CAST_AS_IIrisClosureBlock(pClosureBlock), CAST_AS_IIrisValues(pParameters), CAST_AS_IIrisThreadInfo(pThreadInfo)));
}

IRISLANGLIBRARY_API void IrisDev_ContextEnvironmentSetClosureBlock(CIIrisContextEnvironment pContextEnvironment, CIIrisClosureBlock pBlock)
{
	return IrisDevUtil::ContextEnvironmentSetClosureBlock(CAST_AS_IIrisContextEnvironment(pContextEnvironment), CAST_AS_IIrisClosureBlock(pBlock));
}

IRISLANGLIBRARY_API CIIrisObject IrisDev_GetNativeObjectPointer(CIrisValue ivObj)
{
	return IrisDevUtil::GetNativeObjectPointer(CAST_AS_IrisValue(ivObj));
}

IRISLANGLIBRARY_API int IrisDev_GetObjectID(CIrisValue ivObj)
{
	return IrisDevUtil::GetObjectID(CAST_AS_IrisValue(ivObj));
}

IRISLANGLIBRARY_API CIIrisClass IrisDev_GetClassOfObject(CIrisValue ivObj)
{
	return IrisDevUtil::GetClassOfObject(CAST_AS_IrisValue(ivObj));
}

IRISLANGLIBRARY_API char * IrisDev_GetNameOfClass(CIIrisClass pClass)
{
	return const_cast<char*>(IrisDevUtil::GetNameOfClass(CAST_AS_IIrisClass(pClass)));
}

IRISLANGLIBRARY_API char * IrisDev_GetNameOfModule(CIIrisModule pModule)
{
	return const_cast<char*>(IrisDevUtil::GetNameOfModule(CAST_AS_IIrisModule(pModule)));
}

IRISLANGLIBRARY_API char * IrisDev_GetNameOfInterface(CIIrisInterface pInterface)
{
	return const_cast<char*>(IrisDevUtil::GetNameOfInterface(CAST_AS_IIrisInterface(pInterface)));
}

IRISLANGLIBRARY_API void IrisDev_MarkObject(CIrisValue ivObject)
{
	IrisDevUtil::MarkObject(CAST_AS_IrisValue(ivObject));
}

IRISLANGLIBRARY_API void IrisDev_MarkClosureBlock(CIIrisClosureBlock pClosureBlock)
{
	IrisDevUtil::MarkClosureBlock(CAST_AS_IIrisClosureBlock(pClosureBlock));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_Nil()
{
	return CAST_AS_CIrisValue(IrisDevUtil::Nil());
}

IRISLANGLIBRARY_API CIrisValue IrisDev_False()
{
	return CAST_AS_CIrisValue(IrisDevUtil::False());
}

IRISLANGLIBRARY_API CIrisValue IrisDev_True()
{
	return CAST_AS_CIrisValue(IrisDevUtil::True());
}

IRISLANGLIBRARY_API int IrisDev_RegistClass(char * szPath, CIIrisClass pClass)
{
	return IrisInterpreter::CurrentInterpreter()->RegistClass(szPath, CAST_AS_IIrisClass(pClass));
}

IRISLANGLIBRARY_API int IrisDev_RegistModule(char * szPath, CIIrisModule pModule)
{
	return IrisInterpreter::CurrentInterpreter()->RegistModule(szPath, CAST_AS_IIrisModule(pModule));
}

IRISLANGLIBRARY_API int IrisDev_RegistInterface(char * szPath, CIIrisInterface pInterface)
{
	return IrisInterpreter::CurrentInterpreter()->RegistInterface(szPath, CAST_AS_IIrisInterface(pInterface));
}

IRISLANGLIBRARY_API void IrisDev_ClassAddInstanceMethod(CIIrisClass pClass, char * szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter)
{
	IrisDevUtil::AddInstanceMethod(CAST_AS_IIrisClass(pClass), szMethodName, (IrisNativeFunction)pfFunction, nParamCount, bIsWithVariableParameter ? true : false);
}

IRISLANGLIBRARY_API void IrisDev_ClassAddClassMethod(CIIrisClass pClass, char * szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter)
{
	IrisDevUtil::AddClassMethod(CAST_AS_IIrisClass(pClass), szMethodName, (IrisNativeFunction)pfFunction, nParamCount, bIsWithVariableParameter ? true : false);
}

IRISLANGLIBRARY_API void IrisDev_ModuleAddInstanceMethod(CIIrisModule pModule, char * szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter)
{
	IrisDevUtil::AddInstanceMethod(CAST_AS_IIrisModule(pModule), szMethodName, (IrisNativeFunction)pfFunction, nParamCount, bIsWithVariableParameter ? true : false);
}

IRISLANGLIBRARY_API void IrisDev_ModuleAddClassMethod(CIIrisModule pModule, char * szMethodName, CIrisNativeFunction pfFunction, size_t nParamCount, int bIsWithVariableParameter)
{
	IrisDevUtil::AddClassMethod(CAST_AS_IIrisModule(pModule), szMethodName, (IrisNativeFunction)pfFunction, nParamCount, bIsWithVariableParameter ? true : false);
}

IRISLANGLIBRARY_API void IrisDev_AddGetter(CIIrisClass pClass, char * szInstanceVariableName, CIrisNativeFunction pfFunction)
{
	IrisDevUtil::AddGetter(CAST_AS_IIrisClass(pClass), szInstanceVariableName, (IrisNativeFunction)pfFunction);
}

IRISLANGLIBRARY_API void IrisDev_AddSetter(CIIrisClass pClass, char * szInstanceVariableName, CIrisNativeFunction pfFunction)
{
	IrisDevUtil::AddSetter(CAST_AS_IIrisClass(pClass), szInstanceVariableName, (IrisNativeFunction)pfFunction);
}

IRISLANGLIBRARY_API void IrisDev_ClassAddConstance(CIIrisClass pClass, char * szConstanceName, CIrisValue ivValue)
{
	IrisDevUtil::AddConstance(CAST_AS_IIrisClass(pClass), szConstanceName, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void IrisDev_ModuleAddConstance(CIIrisModule pModule, char * szConstanceName, CIrisValue ivValue)
{
	IrisDevUtil::AddConstance(CAST_AS_IIrisModule(pModule), szConstanceName, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void IrisDev_ClassAddClassVariable(CIIrisClass pClass, char * szVariableName, CIrisValue ivValue)
{
	IrisDevUtil::AddClassVariable(CAST_AS_IIrisClass(pClass), szVariableName, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void IrisDev_ModuleAddClassVariable(CIIrisModule pModule, char * szVariableName, CIrisValue ivValue)
{
	IrisDevUtil::AddClassVariable(CAST_AS_IIrisModule(pModule), szVariableName, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void IrisDev_ClassAddModule(CIIrisClass pClass, CIIrisModule pTargetModule)
{
	IrisDevUtil::AddModule(CAST_AS_IIrisClass(pClass), CAST_AS_IIrisModule(pTargetModule));
}

IRISLANGLIBRARY_API void IrisDev_ModuleAddModule(CIIrisModule pModule, CIIrisModule pTargetModule)
{
	IrisDevUtil::AddModule(CAST_AS_IIrisModule(pModule), CAST_AS_IIrisModule(pTargetModule));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CreateNormalInstance(CIIrisClass pClass, CIIrisValues ivsParams, CIIrisContextEnvironment pContexEnvironment, CIIrisThreadInfo pThreadInfo)
{
	return CAST_AS_CIrisValue(IrisDevUtil::CreateInstance(CAST_AS_IIrisClass(pClass), CAST_AS_IIrisValues(ivsParams), CAST_AS_IIrisContextEnvironment(pContexEnvironment), CAST_AS_IIrisThreadInfo(pThreadInfo)));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CreateStringInstanceByInstantValue(char * szString)
{
	return CAST_AS_CIrisValue(IrisDevUtil::CreateString(szString));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CreateFloatInstanceByInstantValue(double dFloat)
{
	return CAST_AS_CIrisValue(IrisDevUtil::CreateFloat(dFloat));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CreateIntegerInstanceByInstantValue(int nInteger)
{
	return CAST_AS_CIrisValue(IrisDevUtil::CreateInt(nInteger));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_CreateUniqueStringInstanceByUniqueIndex(size_t nIndex)
{
	return CAST_AS_CIrisValue(IrisDevUtil::CreateUniqueStringInstanceByUniqueIndex(nIndex));
}

IRISLANGLIBRARY_API void IrisDev_SetObjectInstanceVariable(CIrisValue & ivObj, char * szInstanceVariableName, CIrisValue ivValue)
{
	IrisDevUtil::SetObjectInstanceVariable(CAST_AS_IrisValue(ivObj), szInstanceVariableName, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API CIrisValue IrisDev_GetObjectInstanceVariable(CIrisValue & ivObj, char * szInstanceVariableName)
{
	return CAST_AS_CIrisValue(IrisDevUtil::GetObjectInstanceVariable(CAST_AS_IrisValue(ivObj), szInstanceVariableName));
}

IRISLANGLIBRARY_API int IrisDev_IrregularHappened(CIIrisThreadInfo pThreadInfo)
{
	return IrisDevUtil::IrregularHappened(CAST_AS_IIrisThreadInfo(pThreadInfo)) ? 1 : 0;
}

IRISLANGLIBRARY_API int IrisDev_FatalErrorHappened(CIIrisThreadInfo pThreadInfo)
{
	return IrisDevUtil::FatalErrorHappened(CAST_AS_IIrisThreadInfo(pThreadInfo)) ? 1 : 0;
}

IRISLANGLIBRARY_API CIIrisValues IrisDev_CreateIrisValuesList(size_t nSize)
{
	return nSize > 0 ? new IrisValues(nSize) : nullptr;
}

IRISLANGLIBRARY_API CIrisValue IrisDev_GetValue(CIIrisValues pValues, size_t nIndex)
{
	return CAST_AS_CIrisValue((static_cast<IrisValues*>(pValues))->GetValue(nIndex));
}

IRISLANGLIBRARY_API void IrisDev_SetValue(CIIrisValues pValues, size_t nIndex, CIrisValue ivValue)
{
	static_cast<IrisValues*>(pValues)->SetValue(nIndex, CAST_AS_IrisValue(ivValue));
}

IRISLANGLIBRARY_API void IrisDev_ReleaseIrisValuesList(CIIrisValues pValues)
{
	delete (IrisValues*)pValues;
}

void _InternFatalErrorMessageFunctionForC(const string& pMessage) {
	g_pfFatalErrorMessageFunctionForC(const_cast<char*>(pMessage.c_str()));
}

bool _InternExitConditionFunctionForC() {
	return g_pfExitConditionFunctionForC();
}

IRISLANGLIBRARY_API int IR_Initialize(PIrisInitializeStructForC pInitializeStruct)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	g_pfExitConditionFunctionForC = pInitializeStruct->m_pfExitConditionFunction;
	g_pfFatalErrorMessageFunctionForC = pInitializeStruct->m_pfFatalErrorMessageFunction;

	IrisFatalErrorHandler::CurrentFatalHandler()->SetFatalErrorMessageFuncton(_InternFatalErrorMessageFunctionForC);
	pInterpreter->SetExitConditionFunction(_InternExitConditionFunctionForC);

	pInterpreter->SetCompiler(pCompiler);

	// Run
	IrisGC::CurrentGC()->SetGCFlag(false);

	if (!pInterpreter->Initialize()) {
		return false;
	}

	IrisGC::CurrentGC()->ResetNextThreshold();
	IrisGC::CurrentGC()->SetGCFlag(true);

	return true;
}

IRISLANGLIBRARY_API int IR_Run()
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	if (!pInterpreter->Run()) {
		return 0;
	}

	return 1;
}

IRISLANGLIBRARY_API int IR_ShutDown()
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	return pInterpreter->ShutDown() ? 1 : 0;
}

IRISLANGLIBRARY_API int IR_LoadScriptFromPath(char * pScriptFilePath)
{
	string strOrgFileName(pScriptFilePath);
	string strDestFileName;
	auto nPos = strOrgFileName.find_last_of(".");
	if (nPos != std::string::npos) {
		strDestFileName.assign(strOrgFileName, 0, nPos);
	}
	strDestFileName += ".irc";

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	if (!pCompiler->LoadScript(pScriptFilePath)) {
		return false;
	}
	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		remove(strDestFileName.c_str());
	}
	return bCompileResult ? 1 : 0;
}

IRISLANGLIBRARY_API int IR_LoadScriptFromVirtualPathAndText(char* pPath, char * pScriptText)
{
	string strDestFileName(pPath);
	strDestFileName += ".irc";

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	if (!pCompiler->LoadScriptFromVirtualPathAndText(pPath, pScriptText)) {
		return false;
	}
	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		remove(strDestFileName.c_str());
	}
	return bCompileResult ? 1 : 0;
}

IRISLANGLIBRARY_API int IR_LoadExtention(char * pExtentionPath)
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	return pInterpreter->LoadExtension(pExtentionPath) ? 1 : 0;
}