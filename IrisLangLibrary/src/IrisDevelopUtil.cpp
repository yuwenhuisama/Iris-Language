#include "IrisCompileConfigure.h"

#include "IrisDevelopUtil.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter/IrisStructure/IrisInterface.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInteger.h"
#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"
#include "IrisInterpreter/IrisNativeClasses/IrisString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClosureBlockBase.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"

#include <string>
using namespace std;
namespace IrisDevUtil {

	IMPLEMENT_CLASS_CHECK(Class)
	IMPLEMENT_CLASS_CHECK(Module)
	IMPLEMENT_CLASS_CHECK(Interface)
	IMPLEMENT_CLASS_CHECK(Object)
	IMPLEMENT_CLASS_CHECK(String)
	IMPLEMENT_CLASS_CHECK(UniqueString)
	IMPLEMENT_CLASS_CHECK(Integer)
	IMPLEMENT_CLASS_CHECK(Float)
	IMPLEMENT_CLASS_CHECK(Array)
	IMPLEMENT_CLASS_CHECK(Hash)
	IMPLEMENT_CLASS_CHECK(Range)
	IMPLEMENT_CLASS_CHECK(Block)

	IrisThreadUniqueInfo * GetCurrentThreadInfo()
	{
		return IrisThreadManager::CurrentThreadManager()->GetThreadInfo(this_thread::get_id());
	}

	bool CurrentThreadIsMainThread()
	{
		return IrisThreadManager::CurrentThreadManager()->IsMainThread();
	}

	bool CheckClass(const IrisValue & ivValue, const char* strClassName)
	{
		auto pClass = static_cast<IrisObject*>(ivValue.GetIrisObject())->GetClass();
		auto pTargetClass = IrisInterpreter::CurrentInterpreter()->GetIrisClass(strClassName);
		return pTargetClass == pClass->GetInternClass();
	}

	void GroanIrregularWithString(const char* strIrregularString)
	{
		auto* pInterpreter = IrisInterpreter::CurrentInterpreter();
		auto* pCompiler = IrisCompiler::CurrentCompiler();
		auto* pInfo = IrisDevUtil::GetCurrentThreadInfo();
		
		auto nLineNumber = pInfo->m_nCurrentLineNumber;
		auto strFileName = pCompiler->GetFileName(pInfo->m_nCurrentFileIndex);

		IrisValue ivLineNumber = IrisDevUtil::CreateInstanceByInstantValue(static_cast<int>(nLineNumber));
		IrisValue ivMsg = IrisDevUtil::CreateString(strIrregularString);

#if IR_USE_STL_STRING
		IrisValue ivFileName = IrisDevUtil::CreateString(strFileName.c_str());
#else
		IrisValue ivFileName = IrisDevUtil::CreateString(strFileName.GetCTypeString());
#endif

		IrisValues ivValues = { ivLineNumber, ivFileName, ivMsg };

		pInterpreter->RegistIrregular(IrisDevUtil::CreateInstance(IrisDevUtil::GetClass("Irregular"), &ivValues, nullptr));
	}

	int GetInt(const IrisValue & ivValue)
	{
		return IrisInteger::GetIntData(ivValue);
	}

	double GetFloat(const IrisValue & ivValue)
	{
		return IrisFloat::GetFloatData(ivValue);
	}

	const char* GetString(const IrisValue & ivValue)
	{
		if (CheckClassIsString(ivValue)) {
			return IrisString::GetString(ivValue).c_str();
		}
		else {
			return IrisUniqueString::GetString(ivValue).c_str();
		}

	}

	IrisValue CallMethod(const IrisValue & ivObj, IIrisValues * pParameters, const char* strMethodName)
	{
		return static_cast<IrisObject*>(ivObj.GetIrisObject())->CallInstanceFunction(strMethodName, nullptr, pParameters, CallerSide::Outside);
	}

	IrisValue CallClassMethod(IIrisClass * pClass, const char * szMethodName, IIrisValues * pParameters)
	{
		return pClass->GetInternClass()->CallClassMethod(szMethodName, nullptr, static_cast<IrisValues*>(pParameters), CallerSide::Outside);
	}

	IrisValue CallClassMethod(IIrisModule * pModule, const char * szMethodName, IIrisValues * pParameters)
	{
		return pModule->GetInternModule()->CallClassMethod(szMethodName, nullptr, static_cast<IrisValues*>(pParameters), CallerSide::Outside);
	}

	IrisValue CreateInt(int nInteger)
	{
		return CreateInstanceByInstantValue(nInteger);
	}

	IrisValue CreateFloat(double dDouble)
	{
		return CreateInstanceByInstantValue(dDouble);
	}

	IrisValue CreateString(const char* nInteger)
	{
		return CreateInstanceByInstantValue(nInteger);
	}

	IIrisClass * GetClass(const char* strClassPathName)
	{
		return IrisInterpreter::CurrentInterpreter()->GetIrisClass(strClassPathName)->GetExternClass();
	}

	IIrisModule * GetModule(const char * strClassPathName)
	{
		return IrisInterpreter::CurrentInterpreter()->GetIrisModule(strClassPathName)->GetExternModule();
	}

	IIrisInterface * GetInterface(const char * strClassPathName)
	{
		return IrisInterpreter::CurrentInterpreter()->GetIrisInterface(strClassPathName)->GetExternInterface();
	}

	bool ObjectIsFixed(const IrisValue& ivObj)
	{
		return static_cast<IrisObject*>(static_cast<IrisObject*>(ivObj.GetIrisObject()))->IsFixed();
	}

	IIrisClosureBlock * GetClosureBlock(IIrisContextEnvironment * pContextEnvironment)
	{
		return static_cast<IrisContextEnvironment*>(pContextEnvironment)->GetClosureBlock();
	}

	IrisValue ExcuteClosureBlock(IIrisClosureBlock * pClosureBlock, IIrisValues* pParameters)
	{
		return static_cast<IrisClosureBlock*>(pClosureBlock)->Excute(pParameters);
	}

	void ContextEnvironmentSetClosureBlock(IIrisContextEnvironment* pContextEnvironment, IIrisClosureBlock* pBlock)
	{
		static_cast<IrisContextEnvironment*>(pContextEnvironment)->SetClosureBlock(pBlock);
	}

	IIrisObject * GetNativeObjectPointer(const IrisValue & ivObj)
	{
		return static_cast<IrisObject*>(ivObj.GetIrisObject())->GetClass()->GetInternClass()->GetClassObject();
	}

	int GetObjectID(const IrisValue & ivObj)
	{
		IrisObject* pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
		int nObjectID = pObject->GetObjectID();
		return nObjectID;
	}

	IIrisClass* GetClassOfObject(const IrisValue & ivObj)
	{
		return static_cast<IrisObject*>(ivObj.GetIrisObject())->GetClass();
	}

	const char * GetNameOfClass(IIrisClass * pClass)
	{
#if IR_USE_STL_STRING
		return pClass->GetInternClass()->GetClassName().c_str();
#else
		return pClass->GetInternClass()->GetClassName().GetCTypeString();
#endif // IR_USE_STL_STRING
	}

	const char * GetNameOfModule(IIrisModule * pModule)
	{
#if IR_USE_STL_STRING
		return pModule->GetInternModule()->GetModuleName().c_str();
#else
		return pModule->GetInternModule()->GetModuleName().GetCTypeString();
#endif // IR_USE_STL_STRING
	}

	const char * GetNameOfInterface(IIrisInterface * pInterface)
	{
#if IR_USE_STL_STRING
		return pInterface->GetInternInterface()->GetInterfaceName().c_str();
#else
		return pInterface->GetInternInterface()->GetInterfaceName().GetCTypeString();
#endif // IR_USE_STL_STRING
	}

	void SetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName, const IrisValue & ivValue)
	{
		bool bResult = false;
		auto& ivResult = static_cast<IrisObject*>(ivObj.GetIrisObject())->GetInstanceValue(szInstanceVariableName, bResult);
		if (bResult) {
			((IrisValue&)ivResult).SetIrisObject(ivValue.GetIrisObject());
		}
		else {
			static_cast<IrisObject*>(ivObj.GetIrisObject())->AddInstanceValue(szInstanceVariableName, ivValue);
		}
	}

	IrisValue GetObjectInstanceVariable(IrisValue & ivObj, char * szInstanceVariableName)
	{
		bool bResult = false;
		auto& ivResult = static_cast<IrisObject*>(ivObj.GetIrisObject())->GetInstanceValue(szInstanceVariableName, bResult);
		return ivResult;
	}

	const char * GetClassName(IIrisClass * pClass)
	{
#if IR_USE_STL_STRING
		return pClass->GetInternClass()->GetClassName().c_str();
#else
		return pClass->GetInternClass()->GetClassName().GetCTypeString();
#endif // IR_USE_STL_STRING
	}

	void MarkObject(const IrisValue& ivObject)
	{
		static_cast<IrisObject*>(ivObject.GetIrisObject())->Mark();
	}

	void MarkClosureBlock(IIrisClosureBlock * pClosureBlock)
	{
		static_cast<IrisClosureBlock*>(pClosureBlock)->Mark();
	}

	void * _InnerGetNativePointer(const IrisValue & ivValue)
	{
		return static_cast<IrisObject*>(ivValue.GetIrisObject())->GetNativeObject();
	}

	void * _InnerGetNativePointer(IIrisObject * pObject)
	{
		return static_cast<IrisObject*>(pObject)->GetNativeObject();
	}

	const IrisValue & Nil()
	{
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}

	const IrisValue & False()
	{
		return IrisInterpreter::CurrentInterpreter()->False();
	}

	const IrisValue & True()
	{
		return IrisInterpreter::CurrentInterpreter()->True();
	}

	void AddInstanceMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		pClass->GetInternClass()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
	}

	void AddClassMethod(IIrisClass * pClass, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		pClass->GetInternClass()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
	}

	void AddInstanceMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		pModule->GetInternModule()->AddInstanceMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
	}

	void AddClassMethod(IIrisModule * pModule, const char * szMethodName, IrisNativeFunction pfFunction, size_t nParamCount, bool bIsWithVariableParameter)
	{
		pModule->GetInternModule()->AddClassMethod(new IrisMethod(szMethodName, pfFunction, nParamCount, bIsWithVariableParameter));
	}

	void AddGetter(IIrisClass * pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
	{
		pClass->GetInternClass()->AddGetter(szInstanceVariableName, pfFunction);
	}

	void AddSetter(IIrisClass* pClass, const char * szInstanceVariableName, IrisNativeFunction pfFunction)
	{
		pClass->GetInternClass()->AddSetter(szInstanceVariableName, pfFunction);
	}

	void AddConstance(IIrisClass * pClass, const char * szConstanceName, const IrisValue & ivValue)
	{
		pClass->GetInternClass()->AddConstance(szConstanceName, ivValue);
	}

	void AddConstance(IIrisModule * pModule, const char * szConstanceName, const IrisValue & ivValue)
	{
		pModule->GetInternModule()->AddConstance(szConstanceName, ivValue);
	}

	void AddClassVariable(IIrisClass * pClass, const char * szVariableName, const IrisValue & ivValue)
	{
		pClass->GetInternClass()->AddClassVariable(szVariableName, ivValue);
	}

	void AddClassVariable(IIrisModule * pClass, const char * szVariableName, const IrisValue & ivValue)
	{
		pClass->GetInternModule()->AddClassVariable(szVariableName, ivValue);
	}

	void AddModule(IIrisClass * pClass, IIrisModule * pTargetModule)
	{
		pClass->GetInternClass()->AddModule(pTargetModule->GetInternModule());
	}

	void AddModule(IIrisModule * pModule, IIrisModule * pTargetModule)
	{
		pModule->GetInternModule()->AddModule(pTargetModule->GetInternModule());
	}

	IIrisClass * GetNativeClass(const IrisValue & ivValue)
	{
		return static_cast<IrisClassBaseTag*>(static_cast<IrisObject*>(ivValue.GetIrisObject())->GetNativeObject())->GetClass()->GetExternClass();
	}

	IIrisModule * GetNativeModule(const IrisValue & ivValue)
	{
		return static_cast<IrisModuleBaseTag*>(static_cast<IrisObject*>(ivValue.GetIrisObject())->GetNativeObject())->GetModule()->GetExternModule();
	}

	IIrisInterface * GetNativeInterface(const IrisValue & ivValue)
	{
		return static_cast<IrisInterfaceBaseTag*>(static_cast<IrisObject*>(ivValue.GetIrisObject())->GetNativeObject())->GetInterface()->GetExternInterface();
	}

	IrisValue CreateInstance(IIrisClass * pClass, IIrisValues * ivsParams, IIrisContextEnvironment * pContexEnvironment)
	{
		return pClass->GetInternClass()->CreateInstance(ivsParams, pContexEnvironment);
	}

	IrisValue CreateInstanceByInstantValue(const char * szString)
	{
		auto pClass = INNER_CLASS_GET_POINTER(String);
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

	IrisValue CreateInstanceByInstantValue(double dFloat)
	{
		auto pClass = INNER_CLASS_GET_POINTER(Float);
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

	IrisValue CreateInstanceByInstantValue(int nInteger)
	{
		auto pClass = INNER_CLASS_GET_POINTER(Integer);
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

	IrisValue CreateInstanceByInstantValue(IIrisClosureBlock * pBlock)
	{
		auto pClass = INNER_CLASS_GET_POINTER(Block);
		IrisValue ivValue;
		IrisObject* pObject = new IrisObject();
		pObject->SetClass(pClass);
		IrisClosureBlockBaseTag *pClosureBlock = new IrisClosureBlockBaseTag(pBlock);
		pObject->SetNativeObject(pClosureBlock);
		ivValue.SetIrisObject(pObject);

		IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
		IrisGC::CurrentGC()->Start();

		// 将新对象保存到堆里
		IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);
		return ivValue;
	}

	IrisValue CreateUniqueStringInstanceByUniqueIndex(size_t nIndex)
	{
		IrisValue ivValue;
		bool bResult = false;
		ivValue = IrisUniqueString::GetUniqueString(nIndex, bResult);
		if (bResult) {
			return ivValue;
		}

		auto pClass = INNER_CLASS_GET_POINTER(UniqueString);

		IrisObject* pObject = new IrisObject();
		pObject->SetClass(pClass);
		pObject->SetPermanent(true);
#if IR_USE_STL_STRING
		IrisUniqueStringTag* pString = new IrisUniqueStringTag(IrisCompiler::CurrentCompiler()->GetUniqueString(nIndex, IrisCompiler::CurrentCompiler()->GetCurrentFileIndex()));
#else
		IrisUniqueStringTag* pString = new IrisUniqueStringTag(IrisCompiler::CurrentCompiler()->GetUniqueString(nIndex, IrisCompiler::CurrentCompiler()->GetCurrentFileIndex()).GetSTLString());
#endif // IR_USE_STL_STRING
		pObject->SetNativeObject(pString);
		ivValue.SetIrisObject(pObject);

		IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
		IrisGC::CurrentGC()->Start();

		// 将新对象保存到堆里
		IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);

		IrisUniqueString::AddUniqueString(nIndex, ivValue);
		return ivValue;
	}

	bool IrregularHappened()
	{
		return IrisInterpreter::CurrentInterpreter()->IrregularHappened();
	}

	bool FatalErrorHappened()
	{
		return IrisInterpreter::CurrentInterpreter()->FatalErrorHappened();
	}
}