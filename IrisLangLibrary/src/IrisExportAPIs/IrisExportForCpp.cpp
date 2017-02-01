#include "IrisExportAPIs/IrisExportForCpp.h"

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

#include <string>
using namespace std;

FatalErrorMessageFunctionForCpp g_pfFatalErrorMessageFunctionForCpp = nullptr;
ExitConditionFunctionForCpp g_pfExitConditionFunctionForCpp = nullptr;

namespace IrisDev {

	#define IMPLEMENT_IRISDEV_CLASS_CHECK(klass) IRISLANGLIBRARY_API bool CheckClassIs##klass(const IrisValue& ivValue) { return IrisDevUtil::CheckClassIs##klass(ivValue);  }

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

	IRISLANGLIBRARY_API void * _InnerGetNativePointer(const IrisValue & ivValue)
	{
		return IrisDevUtil::_InnerGetNativePointer(ivValue);
	}

	IRISLANGLIBRARY_API void * _InnerGetNativePointer(IIrisObject * pObject)
	{
		return IrisDevUtil::_InnerGetNativePointer(pObject);
	}

	IRISLANGLIBRARY_API bool CheckClass(const IrisValue & ivValue, const char* strClassName)
	{
		return IrisDevUtil::CheckClass(ivValue, strClassName);
	}

	IRISLANGLIBRARY_API void GroanIrregularWithString(const char* strIrregularString)
	{
		IrisDevUtil::GroanIrregularWithString(strIrregularString);
	}

	IRISLANGLIBRARY_API int GetInt(const IrisValue & ivValue)
	{
		return IrisDevUtil::GetInt(ivValue);
	}

	IRISLANGLIBRARY_API double GetFloat(const IrisValue & ivValue)
	{
		return IrisDevUtil::GetFloat(ivValue);
	}

	IRISLANGLIBRARY_API const char* GetString(const IrisValue & ivValue)
	{
		return IrisDevUtil::GetString(ivValue);
	}

	IRISLANGLIBRARY_API IrisValue CallMethod(const IrisValue & ivObj, const char* strMethodName, IIrisValues * pParameters)
	{
		return IrisDevUtil::CallMethod(ivObj, pParameters, strMethodName);
	}

	IRISLANGLIBRARY_API IrisValue CallClassClassMethod(IIrisClass * pClass, const char * szMethodName, IIrisValues * pParameters)
	{
		return IrisDevUtil::CallClassMethod(pClass, szMethodName, pParameters);
	}

	IRISLANGLIBRARY_API IrisValue CallClassModuleMethod(IIrisModule * pModule, const char * szMethodName, IIrisValues * pParameters)
	{
		return IrisDevUtil::CallClassMethod(pModule, szMethodName, pParameters);
	}

	IRISLANGLIBRARY_API IIrisClass * GetClass(const char* strClassPathName)
	{
		return IrisDevUtil::GetClass(strClassPathName);
	}

	IRISLANGLIBRARY_API IIrisModule * GetModule(const char * strClassPathName)
	{
		return IrisDevUtil::GetModule(strClassPathName);
	}

	IRISLANGLIBRARY_API IIrisInterface * GetInterface(const char * strClassPathName)
	{
		return IrisDevUtil::GetInterface(strClassPathName);
	}

	IRISLANGLIBRARY_API bool ObjectIsFixed(const IrisValue & ivObj)
	{
		return IrisDevUtil::ObjectIsFixed(ivObj);
	}

	IRISLANGLIBRARY_API IIrisClosureBlock * GetClosureBlock(IIrisContextEnvironment * pContextEnvironment)
	{
		return IrisDevUtil::GetClosureBlock(pContextEnvironment);
	}

	IRISLANGLIBRARY_API IrisValue ExcuteClosureBlock(IIrisClosureBlock * pClosureBlock, IIrisValues * pParameters)
	{
		return IrisDevUtil::ExcuteClosureBlock(pClosureBlock, pParameters);
	}

	IRISLANGLIBRARY_API void ContextEnvironmentSetClosureBlock(IIrisContextEnvironment * pContextEnvironment, IIrisClosureBlock * pBlock)
	{
		return IrisDevUtil::ContextEnvironmentSetClosureBlock(pContextEnvironment, pBlock);
	}

	IRISLANGLIBRARY_API IIrisObject * GetNativeObjectPointer(const IrisValue & ivObj)
	{
		return IrisDevUtil::GetNativeObjectPointer(ivObj);
	}

	IRISLANGLIBRARY_API int GetObjectID(const IrisValue & ivObj)
	{
		return IrisDevUtil::GetObjectID(ivObj);
	}

	IRISLANGLIBRARY_API IIrisClass * GetClassOfObject(const IrisValue & ivObj)
	{
		return IrisDevUtil::GetClassOfObject(ivObj);
	}

	IRISLANGLIBRARY_API const char * GetNameOfClass(IIrisClass * pClass)
	{
		return IrisDevUtil::GetNameOfClass(pClass);
	}

	IRISLANGLIBRARY_API const char * GetNameOfModule(IIrisModule * pModule)
	{
		return IrisDevUtil::GetNameOfModule(pModule);
	}

	IRISLANGLIBRARY_API const char * GetNameOfInterface(IIrisInterface * pInterface)
	{
		return IrisDevUtil::GetNameOfInterface(pInterface);
	}

	IRISLANGLIBRARY_API void MarkObject(const IrisValue & ivObject)
	{
		IrisDevUtil::MarkObject(ivObject);
	}

	IRISLANGLIBRARY_API void MarkClosureBlock(IIrisClosureBlock * pClosureBlock)
	{
		IrisDevUtil::MarkClosureBlock(pClosureBlock);
	}

	IRISLANGLIBRARY_API const IrisValue & Nil()
	{
		return IrisDevUtil::Nil();
	}

	IRISLANGLIBRARY_API const IrisValue & False()
	{
		return IrisDevUtil::False();
	}

	IRISLANGLIBRARY_API const IrisValue & True()
	{
		return IrisDevUtil::True();
	}

	IRISLANGLIBRARY_API bool RegistClass(const char * szPath, IIrisClass * pClass)
	{
		return IrisInterpreter::CurrentInterpreter()->RegistClass(szPath, pClass);
	}

	IRISLANGLIBRARY_API bool RegistModule(const char * szPath, IIrisModule * pModule)
	{
		return IrisInterpreter::CurrentInterpreter()->RegistModule(szPath, pModule);
	}

	IRISLANGLIBRARY_API bool RegistInterface(const char * szPath, IIrisInterface * pInterface)
	{
		return IrisInterpreter::CurrentInterpreter()->RegistInterface(szPath, pInterface);
	}

	IRISLANGLIBRARY_API void AddInstanceMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		IrisDevUtil::AddInstanceMethod(pClass, szMethodName, pfFunction, nParamCount, bIsWithVariableParameter);
	}

	IRISLANGLIBRARY_API void AddClassMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		IrisDevUtil::AddClassMethod(pClass, szMethodName, pfFunction, nParamCount, bIsWithVariableParameter);
	}

	IRISLANGLIBRARY_API void AddInstanceMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		IrisDevUtil::AddInstanceMethod(pModule, szMethodName, pfFunction, nParamCount, bIsWithVariableParameter);
	}

	IRISLANGLIBRARY_API void AddClassMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		IrisDevUtil::AddClassMethod(pModule, szMethodName, pfFunction, nParamCount, bIsWithVariableParameter);
	}

	IRISLANGLIBRARY_API void AddGetter(IIrisClass * pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
	{
		IrisDevUtil::AddGetter(pClass, szInstanceVariableName, pfFunction);
	}

	IRISLANGLIBRARY_API void AddSetter(IIrisClass* pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
	{
		IrisDevUtil::AddSetter(pClass, szInstanceVariableName, pfFunction);
	}

	IRISLANGLIBRARY_API void AddConstance(IIrisClass * pClass, const char * szConstanceName, const IrisValue & ivValue)
	{
		IrisDevUtil::AddConstance(pClass, szConstanceName, ivValue);
	}

	IRISLANGLIBRARY_API void AddConstance(IIrisModule * pModule, const char * szConstanceName, const IrisValue & ivValue)
	{
		IrisDevUtil::AddConstance(pModule, szConstanceName, ivValue);
	}

	IRISLANGLIBRARY_API void AddClassVariable(IIrisClass * pClass, const char * szVariableName, const IrisValue & ivValue)
	{
		IrisDevUtil::AddClassVariable(pClass, szVariableName, ivValue);
	}

	IRISLANGLIBRARY_API void AddClassVariable(IIrisModule * pModule, const char * szVariableName, const IrisValue & ivValue)
	{
		IrisDevUtil::AddClassVariable(pModule, szVariableName, ivValue);
	}

	IRISLANGLIBRARY_API void AddModule(IIrisClass * pClass, IIrisModule * pTargetModule)
	{
		IrisDevUtil::AddModule(pClass, pTargetModule);
	}

	IRISLANGLIBRARY_API void AddModule(IIrisModule * pModule, IIrisModule * pTargetModule)
	{
		IrisDevUtil::AddModule(pModule, pTargetModule);
	}

	IRISLANGLIBRARY_API IrisValue CreateNormalInstance(IIrisClass * pClass, IIrisValues * ivsParams, IIrisContextEnvironment * pContexEnvironment)
	{
		return IrisDevUtil::CreateInstance(pClass, ivsParams, pContexEnvironment);
	}

	IRISLANGLIBRARY_API IrisValue CreateStringInstanceByInstantValue(const char * szString)
	{
		return IrisDevUtil::CreateString(szString);
	}

	IRISLANGLIBRARY_API IrisValue CreateFloatInstanceByInstantValue(double dFloat)
	{
		return IrisDevUtil::CreateFloat(dFloat);
	}

	IRISLANGLIBRARY_API IrisValue CreateIntegerInstanceByInstantValue(int nInteger)
	{
		return IrisDevUtil::CreateInt(nInteger);
	}

	IRISLANGLIBRARY_API IrisValue CreateUniqueStringInstanceByUniqueIndex(size_t nIndex)
	{
		return IrisDevUtil::CreateUniqueStringInstanceByUniqueIndex(nIndex);
	}

	IRISLANGLIBRARY_API void SetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName, const IrisValue & ivValue)
	{
		IrisDevUtil::SetObjectInstanceVariable(ivObj, szInstanceVariableName, ivValue);
	}

	IRISLANGLIBRARY_API IrisValue GetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName)
	{
		return IrisDevUtil::GetObjectInstanceVariable(ivObj, szInstanceVariableName);
	}

	IRISLANGLIBRARY_API bool IrregularHappened()
	{
		return IrisDevUtil::IrregularHappened();
	}

	IRISLANGLIBRARY_API bool FatalErrorHappened()
	{
		return IrisDevUtil::FatalErrorHappened();
	}

	IRISLANGLIBRARY_API IIrisValues * CreateIrisValuesList(size_t nSize)
	{
		return nSize > 0 ? new IrisValues(nSize) : nullptr;
	}

	IRISLANGLIBRARY_API const IrisValue & GetValue(IIrisValues * pValues, size_t nIndex)
	{
		return static_cast<IrisValues*>(pValues)->GetValue(nIndex);
	}

	IRISLANGLIBRARY_API void SetValue(IIrisValues* pValues, size_t nIndex, const IrisValue & ivValue)
	{
		static_cast<IrisValues*>(pValues)->SetValue(nIndex, ivValue);
	}

	IRISLANGLIBRARY_API void ReleaseIrisValuesList(IIrisValues * pValues)
	{
		delete pValues;
	}

}

void _InternFatalErrorMessageFunctionForCpp(const string& pMessage) {
	g_pfFatalErrorMessageFunctionForCpp(pMessage.c_str());
}

bool _InternExitConditionFunctionForCpp() {
	return g_pfExitConditionFunctionForCpp();
}

IRISLANGLIBRARY_API bool IR_Initialize(PIrisInitializeStructForCpp pInitializeStruct)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	g_pfExitConditionFunctionForCpp = pInitializeStruct->m_pfExitConditionFunction;
	g_pfFatalErrorMessageFunctionForCpp = pInitializeStruct->m_pfFatalErrorMessageFunction;

	IrisFatalErrorHandler::CurrentFatalHandler()->SetFatalErrorMessageFuncton(_InternFatalErrorMessageFunctionForCpp);
	pInterpreter->SetExitConditionFunction(_InternExitConditionFunctionForCpp                                                                                                                                                                                                                                                                                                                                                                          );

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

IRISLANGLIBRARY_API bool IR_Run()
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	if (!pInterpreter->Run()) {
		return false;
	}

	return true;
}

IRISLANGLIBRARY_API bool IR_ShutDown()
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	return pInterpreter->ShutDown();
}

IRISLANGLIBRARY_API bool IR_LoadScriptFromPath(const char * pScriptFilePath)
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
	return bCompileResult;
}

IRISLANGLIBRARY_API bool IR_LoadScriptFromVirtualPathAndText(const char* pPath, const  char * pScriptText)
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
	return bCompileResult;
}

IRISLANGLIBRARY_API bool IR_LoadExtention(const char * pExtentionPath)
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	return pInterpreter->LoadExtension(pExtentionPath);
}