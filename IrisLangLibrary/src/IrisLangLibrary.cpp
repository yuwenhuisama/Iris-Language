// IrisLangLibrary.cpp : 定义 DLL 应用程序的导出函数。
//

#include "stdafx.h"
#include "IrisLangLibrary.h"

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

IrisDev_FatalErrorMessageFunction g_pfFatalErrorMessageFunction = nullptr;
IrisDev_ExitConditionFunction g_pfExitConditionFunction = nullptr;

IRISLANGLIBRARY_API bool IrisDev_CheckClass(const IrisValue & ivValue, const char* strClassName)
{
	auto pClass = ((IrisValue&)ivValue).GetIrisObject()->GetClass();
	auto& strName = pClass->GetInternClass()->GetClassName();
	return strName == strClassName;
}

IRISLANGLIBRARY_API void IrisDev_GroanIrregularWithString(const char* strIrregularString)
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	IrisValue ivValue = IrisDev_CreateStringInstanceByInstantValue(strIrregularString);
	pInterpreter->RegistIrregular(ivValue);
}

IRISLANGLIBRARY_API int IrisDev_GetInt(const IrisValue & ivValue)
{
	return IrisInteger::GetIntData((IrisValue&)ivValue);
}

IRISLANGLIBRARY_API double IrisDev_GetFloat(const IrisValue & ivValue)
{
	return IrisFloat::GetFloatData((IrisValue&)ivValue);
}

IRISLANGLIBRARY_API const char* IrisDev_GetString(const IrisValue & ivValue)
{
	if (IrisDev_CheckClass(ivValue, "String")) {
		return IrisString::GetString((IrisValue&)ivValue).c_str();
	}
	else {
		return IrisUniqueString::GetString((IrisValue&)ivValue).c_str();
	}

}

IRISLANGLIBRARY_API IrisValue IrisDev_CallMethod(const IrisValue & ivObj, IIrisValues * pParameters, const char* strMethodName)
{
	return static_cast<IrisObject*>(((IrisValue&)ivObj).GetIrisObject())->CallInstanceFunction(strMethodName, nullptr, pParameters, CallerSide::Outside);
}

IRISLANGLIBRARY_API IIrisClass * IrisDev_GetClass(const char* strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisClass(strClassPathName)->GetExternClass();
}

IRISLANGLIBRARY_API IIrisModule * IrisDev_GetModule(const char * strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisModule(strClassPathName)->GetExternModule();
}

IRISLANGLIBRARY_API IIrisInterface * IrisDev_GetInterface(const char * strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisInterface(strClassPathName)->GetExternInterface();
}

IRISLANGLIBRARY_API const IrisValue & IrisDev_Nil()
{
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IRISLANGLIBRARY_API const IrisValue & IrisDev_False()
{
	return IrisInterpreter::CurrentInterpreter()->False();
}

IRISLANGLIBRARY_API const IrisValue & IrisDev_True()
{
	return IrisInterpreter::CurrentInterpreter()->True();
}

IRISLANGLIBRARY_API bool IrisDev_RegistClass(const char * szPath, IIrisClass * pClass)
{
	return IrisInterpreter::CurrentInterpreter()->RegistClass(szPath, pClass);
}

IRISLANGLIBRARY_API bool IrisDev_RegistModule(const char * szPath, IIrisModule * pModule)
{
	return IrisInterpreter::CurrentInterpreter()->RegistModule(szPath, pModule);
}

IRISLANGLIBRARY_API bool IrisDev_RegistInterface(const char * szPath, IIrisInterface * pInterface)
{
	return IrisInterpreter::CurrentInterpreter()->RegistInterface(szPath, pInterface);
}

IRISLANGLIBRARY_API void IrisDev_AddInstanceMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pClass->GetInternClass()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

IRISLANGLIBRARY_API void IrisDev_AddClassMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pClass->GetInternClass()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

IRISLANGLIBRARY_API void IrisDev_AddInstanceMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pModule->GetInternModule()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

IRISLANGLIBRARY_API void IrisDev_AddClassMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pModule->GetInternModule()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

IRISLANGLIBRARY_API void IrisDev_AddGetter(IIrisClass * pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
{
	pClass->GetInternClass()->AddGetter(szInstanceVariableName, pfFunction);
}

IRISLANGLIBRARY_API void IrisDev_AddSetter(IIrisClass* pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
{
	pClass->GetInternClass()->AddSetter(szInstanceVariableName, pfFunction);
}

IRISLANGLIBRARY_API void IrisDev_AddConstance(IIrisClass * pClass, const char * szConstanceName, const IrisValue & ivValue)
{
	pClass->GetInternClass()->AddConstance(szConstanceName, ivValue);
}

IRISLANGLIBRARY_API void IrisDev_AddConstance(IIrisModule * pModule, const char * szConstanceName, const IrisValue & ivValue)
{
	pModule->GetInternModule()->AddConstance(szConstanceName, ivValue);
}

IRISLANGLIBRARY_API void IrisDev_AddClassVariable(IIrisClass * pClass, const char * szVariableName, const IrisValue & ivValue)
{
	pClass->GetInternClass()->AddClassVariable(szVariableName, ivValue);
}

IRISLANGLIBRARY_API void IrisDev_AddClassVariable(IIrisModule * pClass, const char * szVariableName, const IrisValue & ivValue)
{
	pClass->GetInternModule()->AddClassVariable(szVariableName, ivValue);
}

IRISLANGLIBRARY_API void IrisDev_AddModule(IIrisClass * pClass, IIrisModule * pTargetModule)
{
	pClass->GetInternClass()->AddModule(pTargetModule->GetInternModule());
}

IRISLANGLIBRARY_API void IrisDev_AddModule(IIrisModule * pModule, IIrisModule * pTargetModule)
{
	pModule->GetInternModule()->AddModule(pTargetModule->GetInternModule());
}

IRISLANGLIBRARY_API IrisValue IrisDev_CreateNormalInstance(IIrisClass * pClass, IIrisValues * ivsParams, IIrisContextEnvironment * pContexEnvironment)
{
	return pClass->GetInternClass()->CreateInstance(ivsParams, pContexEnvironment);
}

IRISLANGLIBRARY_API IrisValue IrisDev_CreateStringInstanceByInstantValue(const char * szString)
{
	auto pClass = IrisDev_GetClass("String");
	IrisValue ivValue;
	IrisObject* pObject = new IrisObject();
	pObject->SetClass(pClass);
	IrisStringTag* pString = new IrisStringTag(szString);
	pObject->SetNativeObject(pString);
	ivValue.SetIrisObject(pObject);

	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	IrisGC::CurrentGC()->Start();

	// 将新对象保存到堆里
	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	return ivValue;
}

IRISLANGLIBRARY_API IrisValue IrisDev_CreateFloatInstanceByInstantValue(double dFloat)
{
	auto pClass = IrisDev_GetClass("Float");
	IrisValue ivValue;
	IrisObject* pObject = new IrisObject();
	pObject->SetClass(pClass);
	IrisFloatTag* pFloat = new IrisFloatTag(dFloat);
	pObject->SetNativeObject(pFloat);
	ivValue.SetIrisObject(pObject);

	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	IrisGC::CurrentGC()->Start();

	// 将新对象保存到堆里
	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	return ivValue;
}

IRISLANGLIBRARY_API IrisValue IrisDev_CreateIntegerInstanceByInstantValue(int nInteger)
{
	auto pClass = IrisDev_GetClass("Integer");
	IrisValue ivValue;
	IrisObject* pObject = new IrisObject();
	pObject->SetClass(pClass);
	IrisIntegerTag* pInteger = new IrisIntegerTag(nInteger);
	pObject->SetNativeObject(pInteger);
	ivValue.SetIrisObject(pObject);

	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	IrisGC::CurrentGC()->Start();

	// 将新对象保存到堆里
	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
	return ivValue;
}

IRISLANGLIBRARY_API IrisValue IrisDev_CreateUniqueStringInstanceByUniqueIndex(size_t nIndex)
{
	IrisValue ivValue;
	bool bResult = false;
	ivValue = IrisUniqueString::GetUniqueString(nIndex, bResult);
	if (bResult) {
		return ivValue;
	}

	auto pClass = IrisDev_GetClass("UniqueString");

	IrisObject* pObject = new IrisObject();
	pObject->SetClass(pClass);
	pObject->SetPermanent(true);
	IrisUniqueStringTag* pString = new IrisUniqueStringTag(IrisCompiler::CurrentCompiler()->GetUniqueString(nIndex, IrisCompiler::CurrentCompiler()->GetCurrentFileIndex()));
	pObject->SetNativeObject(pString);
	ivValue.SetIrisObject(pObject);

	IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	IrisGC::CurrentGC()->Start();

	// 将新对象保存到堆里
	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);

	IrisUniqueString::AddUniqueString(nIndex, ivValue);
	return ivValue;
}

IRISLANGLIBRARY_API void IrisDev_SetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName, const IrisValue & ivValue)
{
	bool bResult = false;
	auto& ivResult =  ivObj.GetIrisObject()->GetInstanceValue(szInstanceVariableName, bResult);
	if (bResult) {
		((IrisValue&)ivResult).SetIrisObject(ivResult.GetIrisObject());
	}
	else {
		ivObj.GetIrisObject()->AddInstanceValue(szInstanceVariableName, ivValue);
	}
}

IRISLANGLIBRARY_API IrisValue IrisDev_GetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName)
{
	bool bResult = false;
	auto& ivResult = ivObj.GetIrisObject()->GetInstanceValue(szInstanceVariableName, bResult);
	return ivResult;
}

IRISLANGLIBRARY_API bool IrisDev_IrregularHappened()
{
	return IrisInterpreter::CurrentInterpreter()->IrregularHappened();
}

IRISLANGLIBRARY_API bool IrisDev_FatalErrorHappened()
{
	return IrisInterpreter::CurrentInterpreter()->FatalErrorHappened();
}

void _InternFatalErrorMessageFunction(const string& pMessage) {
	g_pfFatalErrorMessageFunction((char*)pMessage.c_str());
}

bool _InternExitConditionFunction() {
	return g_pfExitConditionFunction() ? true : false;
}

IRISLANGLIBRARY_API bool IR_Initialize(PIrisInitializeStruct pInitializeStruct)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	g_pfExitConditionFunction = pInitializeStruct->m_pfExitConditionFunction;
	g_pfFatalErrorMessageFunction = pInitializeStruct->m_pfFatalErrorMessageFunction;
	
	IrisFatalErrorHandler::CurrentFatalHandler()->SetFatalErrorMessageFuncton(_InternFatalErrorMessageFunction);
	pInterpreter->SetExitConditionFunction(_InternExitConditionFunction);

	return true;
}

IRISLANGLIBRARY_API bool IR_Run()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	pInterpreter->SetCompiler(pCompiler);

	// Run
	IrisGC::CurrentGC()->SetGCFlag(false);

	if (!pInterpreter->Initialize()) {
		return false;
	}

	IrisGC::CurrentGC()->ResetNextThreshold();
	IrisGC::CurrentGC()->SetGCFlag(true);

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

IRISLANGLIBRARY_API bool IR_LoadScriptFromPath(char * pScriptFilePath)
{
	string strOrgFileName(pScriptFilePath);
	string strDestFileName;
	auto nPos = strOrgFileName.find_last_of(".");
	if (nPos != std::string::npos) {
		strDestFileName.assign(strOrgFileName, 0, nPos);
	}
	strDestFileName += ".irc";

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->LoadScript(pScriptFilePath);
	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		remove(strDestFileName.c_str());
	}
	return bCompileResult;
}

IRISLANGLIBRARY_API bool IR_LoadScriptFromVirtualPathAndText(char* pPath, char * pScriptText)
{
	string strDestFileName(pPath);
	strDestFileName += ".irc";

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->LoadScriptFromVirtualPathAndText(pPath, pScriptText);
	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		remove(strDestFileName.c_str());
	}
	return bCompileResult;
}

IRISLANGLIBRARY_API bool IR_LoadExtention(char * pExtentionPath)
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	return pInterpreter->LoadExtension(pExtentionPath);
}