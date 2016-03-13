#include "IrisDevelopUtil.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInteger.h"
#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"
#include "IrisInterpreter/IrisNativeClasses/IrisString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.H"

#include <string>
using namespace std;

bool IrisDev_CheckClass(IrisValue & ivValue, const char* strClassName)
{
	auto pClass = ivValue.GetIrisObject()->GetClass();
	auto& strName = pClass->GetInternClass()->GetClassName();
	return strName == strClassName;
}

void IrisDev_GroanIrregularWithString(const char* strIrregularString)
{
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	IrisValue ivValue = IrisDev_CreateInstanceByInstantValue(strIrregularString);
	pInterpreter->RegistIrregular(ivValue);
}

int IrisDev_GetInt(IrisValue & ivValue)
{
	return IrisInteger::GetIntData(ivValue);
}

double IrisDev_GetFloat(IrisValue & ivValue)
{
	return IrisFloat::GetFloatData(ivValue);
}

const char* IrisDev_GetString(IrisValue & ivValue)
{
	if (IrisDev_CheckClass(ivValue, "String")) {
		return IrisString::GetString(ivValue).c_str();
	}
	else {
		return IrisUniqueString::GetString(ivValue).c_str();
	}
	
}

IrisValue IrisDev_CallMethod(IrisValue & ivObj, IIrisValues * pParameters, const char* strMethodName)
{
	if (ivObj.GetIrisObject()->GetClass()->GetInternClass()->IsClassClass()) {
		return IrisDev_GetNativePointer<IrisClass*>(ivObj)->CallClassMethod(strMethodName, nullptr, static_cast<IrisValues*>(pParameters), CallerSide::Outside);
	}
	else if (ivObj.GetIrisObject()->GetClass()->GetInternClass()->IsModuleClass()) {
		return IrisDev_GetNativePointer<IrisModule*>(ivObj)->CallClassMethod(strMethodName, nullptr, static_cast<IrisValues*>(pParameters), CallerSide::Outside);
	}
	else {
		return static_cast<IrisObject*>(ivObj.GetIrisObject())->CallInstanceFunction(strMethodName, nullptr, pParameters, CallerSide::Outside);
	}
}

IrisValue IrisDev_CreateInt(int nInteger)
{
	return IrisDev_CreateInstanceByInstantValue(nInteger);
}

IrisValue IrisDev_CreateFloat(double dDouble)
{
	return IrisDev_CreateInstanceByInstantValue(dDouble);
}

IrisValue IrisDev_CreateString(const char* nInteger)
{
	return IrisDev_CreateInstanceByInstantValue(nInteger);
}

IIrisClass * IrisDev_GetClass(const char* strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisClass(strClassPathName)->GetExternClass();
}

IIrisModule * IrisDev_GetModule(const char * strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisModule(strClassPathName)->GetExternModule();
}

IIrisInterface * IrisDev_GetInterface(const char * strClassPathName)
{
	return IrisInterpreter::CurrentInterpreter()->GetIrisInterface(strClassPathName)->GetExternInterface();
}

const IrisValue & IrisDev_Nil()
{
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

const IrisValue & IrisDev_False()
{
	return IrisInterpreter::CurrentInterpreter()->False();
}

const IrisValue & IrisDev_True()
{
	return IrisInterpreter::CurrentInterpreter()->True();
}

void IrisDev_AddInstanceMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pClass->GetInternClass()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

void IrisDev_AddClassMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pClass->GetInternClass()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

void IrisDev_AddInstanceMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pModule->GetInternModule()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

void IrisDev_AddClassMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
{
	pModule->GetInternModule()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
}

void IrisDev_AddGetter(IIrisClass * pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
{
	pClass->GetInternClass()->AddGetter(szInstanceVariableName, pfFunction);
}

void IrisDev_AddSetter(IIrisClass* pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
{
	pClass->GetInternClass()->AddSetter(szInstanceVariableName, pfFunction);
}

void IrisDev_AddConstance(IIrisClass * pClass, const char * szConstanceName, const IrisValue & ivValue)
{
	pClass->GetInternClass()->AddConstance(szConstanceName, ivValue);
}

void IrisDev_AddConstance(IIrisModule * pModule, const char * szConstanceName, const IrisValue & ivValue)
{
	pModule->GetInternModule()->AddConstance(szConstanceName, ivValue);
}

void IrisDev_AddClassVariable(IIrisClass * pClass, const char * szVariableName, const IrisValue & ivValue)
{
	pClass->GetInternClass()->AddClassVariable(szVariableName, ivValue);
}

void IrisDev_AddClassVariable(IIrisModule * pClass, const char * szVariableName, const IrisValue & ivValue)
{
	pClass->GetInternModule()->AddClassVariable(szVariableName, ivValue);
}

void IrisDev_AddModule(IIrisClass * pClass, IIrisModule * pTargetModule)
{
	pClass->GetInternClass()->AddModule(pTargetModule->GetInternModule());
}

void IrisDev_AddModule(IIrisModule * pModule, IIrisModule * pTargetModule)
{
	pModule->GetInternModule()->AddModule(pTargetModule->GetInternModule());
}

IrisValue IrisDev_CreateInstance(IIrisClass * pClass, IIrisValues * ivsParams, IIrisContextEnvironment * pContexEnvironment)
{
	return pClass->GetInternClass()->CreateInstance(ivsParams, pContexEnvironment);
}

IrisValue IrisDev_CreateInstanceByInstantValue(const char * szString)
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

IrisValue IrisDev_CreateInstanceByInstantValue(double dFloat)
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

IrisValue IrisDev_CreateInstanceByInstantValue(int nInteger)
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

//IrisValue IrisDev_CreateFloatInstanceByInstantValue(char * szLiterral)
//{
//	return IrisValue();
//}
//
//IrisValue IrisDev_CreateIntegerInstanceByInstantValue(char * szLiterral)
//{
//	return IrisValue();
//}
//
//IrisValue IrisDev_CreateStringInstanceByInstantValue(char * szLiterral)
//{
//	return IrisValue();
//}

IrisValue IrisDev_CreateUniqueStringInstanceByUniqueIndex(size_t nIndex)
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

bool IrisDev_IrregularHappened()
{
	return IrisInterpreter::CurrentInterpreter()->IrregularHappened();
}

bool IrisDev_FatalErrorHappened()
{
	return IrisInterpreter::CurrentInterpreter()->FatalErrorHappened();
}
