#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisFatalErrorHandler.h"


void IrisModule::_SearchConstance(const string& strConstanceName, IrisModule* pCurModule, IrisValue** pValue) {
	//出口
	if (pCurModule == nullptr) {
		return;
	}

	//decltype(pCurModule->m_hsConstances)::iterator iCons;
	//if ((iCons = pCurModule->m_hsConstances.find(strConstanceName)) != pCurModule->m_hsConstances.end()) {
	//	*pValue = &iCons->second;
	//	return;
	//}

	bool bResult = false;
	auto& ivResultValue = pCurModule->GetCurrentModuleConstance(strConstanceName, bResult);
	if (bResult) {
		*pValue = (IrisValue*)&ivResultValue;
	}

	for (auto module : pCurModule->m_hsModules) {
		_SearchConstance(strConstanceName, module.second, pValue);
		if (pValue)
			return;
	}
}
void IrisModule::_SearchClassVariable(const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue){
	//出口
	if (pCurModule == nullptr) {
		return;
	}

	//decltype(pCurModule->m_hsClassVariables)::iterator iVariable;
	//if ((iVariable = pCurModule->m_hsClassVariables.find(strVariableName)) != pCurModule->m_hsClassVariables.end()) {
	//	*pValue = &iVariable->second;
	//	return;
	//}

	bool bResult = false;
	auto& ivResultValue = pCurModule->GetCurrentModuleClassVariable(strVariableName, bResult);
	if (bResult) {
		*pValue = (IrisValue*)&ivResultValue;
	}

	for (auto module : pCurModule->m_hsModules) {
		_SearchClassVariable(strVariableName, module.second, pValue);
		if (*pValue)
			return;
	}
}

const IrisValue& IrisModule::SearchConstance(const string& strConstName, bool& bResult) {
	bResult = true;
	IrisValue* pValue = nullptr;

	_SearchConstance(strConstName, this, &pValue);
	if (pValue){
		return *pValue;
	}
	else{
		pValue = (IrisValue*)&IrisInterpreter::CurrentInterpreter()->GetOtherValue(strConstName, bResult);
		if (bResult) {
			return *pValue;
		}
		bResult = false;
		return IrisDevUtil::Nil();
	}
}

void IrisModule::AddModule(IrisModule* pModule) {
	//m_hsModules.insert(_ModulePair(pModule->m_strModuleName, pModule));

	string strFullPath = "";
	IrisModule* pTmpModule = pModule->m_pUpperModule;
	while (pTmpModule) {
		strFullPath = pTmpModule->GetModuleName() + "::" + strFullPath;
		pTmpModule = pTmpModule->m_pUpperModule;
	}
	strFullPath += pModule->GetModuleName();

	m_iwlModuleAddingWLLock.WriteLock();

	if (m_hsModules.find(strFullPath) != m_hsModules.end()) {
		m_hsModules[strFullPath] = pModule;
	}
	else {
		m_hsModules.insert(_ModulePair(strFullPath, pModule));
	}

	m_iwlModuleAddingWLLock.WriteUnlock();
}

const IrisValue& IrisModule::SearchClassVariable(const string& strClassVariableName, bool& bResult) {
	bResult = true;
	IrisValue* pValue = nullptr;

	_SearchClassVariable(strClassVariableName, this, &pValue);

	if (pValue){
		return *pValue;
	}
	else{
		bResult = false;
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
}

void IrisModule::_SearchModuleMethod(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {

	// 出口
	if (*ppMethod)
		return;

	//decltype(pCurModule->m_hsClassMethods)::iterator iMethod;
	//if ((iMethod = pCurModule->m_hsClassMethods.find(strFunctionName)) != pCurModule->m_hsClassMethods.end()) {
	//	*ppMethod = iMethod->second;
	//	return;
	//}

	IrisMethod* pMethod = nullptr;
	pMethod = pCurModule->GetCurrentModuleMethod(strFunctionName);
	if (pMethod) {
		*ppMethod = pMethod;
		return;
	}

	for (auto module : pCurModule->m_hsModules) {
		_SearchModuleMethod(strFunctionName, module.second, ppMethod);
		if (*ppMethod)
			return;
	}
}

IrisValue IrisModule::CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber, int nBelongingFileIndex) {

	return static_cast<IrisObject*>(m_pModuleObject)->CallInstanceFunction(strMethodName, pContextEnvironment, ivParameters, eSide, nLineNumber, nBelongingFileIndex);

	//IrisMethod* pMethod = nullptr;
	//bool bIsCrrentModuleMethod = false;

	//if (!strBelongingFileName) {
	//	strBelongingFileName = &IrisDevUtil::GetCurrentThreadInfo()->m_strCurrentFileName;
	//}

	//decltype(m_hsClassMethods)::iterator iMethod;
	//if ((iMethod = m_hsClassMethods.find(strMethodName)) != m_hsClassMethods.end()) {
	//	pMethod = iMethod->second;
	//	bIsCrrentModuleMethod = true;
	//}
	//else {
	//	for (auto module : m_hsModules) {
	//		_SearchModuleMethod(strMethodName, module.second, &pMethod);
	//		if (pMethod) {
	//			break;
	//		}
	//	}
	//}

	//if (pMethod) {
	//	// 内部调用
	//	if (eSide == CallerSide::Inside) {
	//		// 当前模块调用没有限制
	//		if (bIsCrrentModuleMethod) {
	//			return pMethod->Call(IrisValue::WrapObjectPointerToIrisValue(m_pModuleObject), pContextEnvironment, ivParameters, nLineNumber, nBelongingFileIndex);
	//		}
	//		// 调用其他模块的Personal方法
	//		else {
	//			if (pMethod->GetAuthority() == IrisMethod::MethodAuthority::Personal) {
	//				// **Error**
	//				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, nLineNumber, nBelongingFileIndex, "Method of " + strMethodName + " is PERSONAL and cannot be called from module " + m_strModuleName + ".");
	//				return IrisInterpreter::CurrentInterpreter()->Nil();
	//			}
	//			else {
	//				return pMethod->Call(IrisValue::WrapObjectPointerToIrisValue(m_pModuleObject), pContextEnvironment, ivParameters, nLineNumber, nBelongingFileIndex);
	//			}
	//		}
	//	}
	//	// 外部调用
	//	else {
	//		// 只能调用EveryOne的方法
	//		// 禁止调用
	//		if (pMethod->GetAuthority() != IrisMethod::MethodAuthority::Everyone) {
	//			// **Error**
	//			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, nLineNumber, nBelongingFileIndex, "Method of " + strMethodName + " is not EVERYONE and cannot be called outside the module " + m_strModuleName + ".");
	//			return IrisInterpreter::CurrentInterpreter()->Nil();
	//		}
	//		else {
	//			return pMethod->Call(IrisValue::WrapObjectPointerToIrisValue(m_pModuleObject), pContextEnvironment, ivParameters, nLineNumber, nBelongingFileIndex);
	//		}
	//	}
	//}
	//// **Error**
	//IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, nLineNumber, nBelongingFileIndex, "Method of " + strMethodName + " not found in module " + m_strModuleName + ".");
	//return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisModule::IrisModule(const string& strModuleName, IrisModule* pUpperModule) : m_strModuleName(strModuleName), m_pUpperModule(pUpperModule) {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Module")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(ivValue.GetIrisObject())->SetModule(this);
	m_pModuleObject = ivValue.GetIrisObject();
}

void IrisModule::ResetAllMethodsObject() {
	//for (auto classmethod : m_hsClassMethods) {
		//classmethod.second->ResetObject();
	//}

	auto& hsInsMethods = static_cast<IrisObject*>(m_pModuleObject)->m_mpInstanceMethods;

	for (auto& method : hsInsMethods) {
		method.second->ResetObject();
	}

	for (auto instancemethod : m_hsInstanceMethods) {
		instancemethod.second->ResetObject();
	}
}

IrisMethod* IrisModule::GetCurrentModuleMethod(const string& strMethodName) {
	IrisMethod* pResult = nullptr;

	decltype(m_hsInstanceMethods)::iterator iMethod;
	m_iwlInstanceMethodWLLock.ReadLock();
	if ((iMethod = m_hsInstanceMethods.find(strMethodName)) != m_hsInstanceMethods.end()) {
		//	return nullptr;
		pResult = iMethod->second;
	}
	else {
		pResult = nullptr;
	}
	m_iwlInstanceMethodWLLock.ReadUnlock();
	return pResult;
}

const IrisValue & IrisModule::GetCurrentModuleClassVariable(const string& strVariableName, bool& bResult)
{
	decltype(m_hsClassVariables)::iterator iVariable;
	m_iwlModuleClassVariableWLLock.ReadLock();
	if ((iVariable = m_hsClassVariables.find(strVariableName)) != m_hsClassVariables.end()){
		auto& ivResult = iVariable->second;
		bResult = true;
		m_iwlModuleClassVariableWLLock.ReadUnlock();
		return ivResult;
	}
	else {
		bResult = false;
		m_iwlModuleClassVariableWLLock.ReadUnlock();
		return IrisDevUtil::Nil();
	}
}

const IrisValue & IrisModule::GetCurrentModuleConstance(const string& strConstanceName, bool& bResult)
{
	decltype(m_hsConstances)::iterator iVariable;
	m_iwlModuleConstanceWLLock.ReadLock();
	if ((iVariable = m_hsConstances.find(strConstanceName)) != m_hsConstances.end()) {
		auto& ivResult = iVariable->second;
		bResult = true;
		m_iwlModuleConstanceWLLock.ReadUnlock();
		return ivResult;
	}
	else {
		m_iwlModuleConstanceWLLock.ReadUnlock();
		bResult = false;
		return IrisDevUtil::Nil();
	}
}

IrisModule::~IrisModule() {
	for (auto iter : m_hsInstanceMethods) {
		delete iter.second;
	}
	//for (auto iter : m_hsClassMethods) {
		//delete iter.second;
	//}
	delete m_pExternModule;
}

void IrisModule::ClearInvolvingModules() {
	m_iwlModuleAddingWLLock.WriteLock();
	m_hsModules.clear();
	m_iwlModuleAddingWLLock.WriteUnlock();
}

void IrisModule::ClearJointedInterfaces()
{
	m_iwlInterfaceAddingWLLock.WriteLock();
	m_hsModules.clear();
	m_iwlInterfaceAddingWLLock.WriteUnlock();
}

void IrisModule::AddConstance(const string & strConstName, const IrisValue & ivInitialValue) {
	m_iwlModuleConstanceWLLock.WriteLock();
	m_hsConstances.insert(_VariablePair(strConstName, ivInitialValue));
	m_iwlModuleConstanceWLLock.WriteUnlock();
}

void IrisModule::AddClassMethod(IrisMethod * pMethod) {
	//m_hsClassMethods.insert(_MethodPair(pMethod->m_strMethodName, pMethod));
	 static_cast<IrisObject*>(m_pModuleObject)->AddSingleInstanceMethod(pMethod);
}

void IrisModule::AddInstanceMethod(IrisMethod * pMethod) {
	//m_hsInstanceMethods.insert(_MethodPair(pMethod->m_strMethodName, pMethod));
	decltype(m_hsInstanceMethods)::iterator iMethod;
	m_iwlInstanceMethodWLLock.WriteLock();
	if ((iMethod = m_hsInstanceMethods.find(pMethod->GetMethodName())) != m_hsInstanceMethods.end()) {
		delete iMethod->second;
		m_hsInstanceMethods[pMethod->GetMethodName()] = pMethod;
	}
	else {
		m_hsInstanceMethods.insert(_MethodPair(pMethod->GetMethodName(), pMethod));
	}
	m_iwlInstanceMethodWLLock.WriteUnlock();
}
void IrisModule::AddClassVariable(const string & strClassVariableName) {
	m_iwlModuleClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, IrisInterpreter::CurrentInterpreter()->Nil()));
	m_iwlModuleClassVariableWLLock.WriteUnlock();
}

void IrisModule::AddClassVariable(const string & strClassVariableName, const IrisValue & ivInitialValue) {
	m_iwlModuleClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, ivInitialValue));
	m_iwlModuleClassVariableWLLock.WriteUnlock();
}

void IrisModule::AddClass(IrisClass * pClass) {
	//decltype(m_hsClasses)::iterator iClass;
	m_iwlClassAddingWLLock.WriteLock();
	if (m_hsClasses.find(pClass->GetClassName()) == m_hsClasses.end()) {
		m_hsClasses.insert(_ClassPair(pClass->GetClassName(), pClass));
	}
	else {
		m_hsClasses[pClass->GetClassName()] = pClass;
	}
	m_iwlClassAddingWLLock.WriteUnlock();
}

void IrisModule::AddInvolvedInterface(IrisInterface * pInterface) {
	m_iwlInterfaceAddingWLLock.WriteLock();
	decltype(m_hsClasses)::iterator iInterface;
	if (m_hsInvolvedInterfaces.find(pInterface->GetInterfaceName()) == m_hsInvolvedInterfaces.end()) {
		m_hsInvolvedInterfaces.insert(_InterfacePair(pInterface->GetInterfaceName(), pInterface));
	}
	else {
		m_hsInvolvedInterfaces[pInterface->GetInterfaceName()] = pInterface;
	}
	m_iwlInterfaceAddingWLLock.WriteUnlock();
}

IrisClass * IrisModule::GetClass(const string & strClassName) {
	IrisClass* pClass = nullptr;
	m_iwlClassAddingWLLock.ReadLock();
	decltype(m_hsClasses)::iterator iClass;
	if ((iClass = m_hsClasses.find(strClassName)) != m_hsClasses.end()) {
		pClass = iClass->second;
	}
	m_iwlClassAddingWLLock.ReadUnlock();
	return pClass;
}

IrisInterface * IrisModule::GetInterface(const string & strInterfaceName) {
	IrisInterface* pInterface = nullptr;
	m_iwlInterfaceAddingWLLock.ReadLock();
	decltype(m_hsInvolvedInterfaces)::iterator iInterface;
	if ((iInterface = m_hsInvolvedInterfaces.find(strInterfaceName)) != m_hsInvolvedInterfaces.end()) {
		pInterface = iInterface->second;
	}
	m_iwlInterfaceAddingWLLock.ReadUnlock();
	return pInterface;
}

IrisMethod * IrisModule::GetModuleInstanceMethod(const string& strMethodName)
{
	IrisMethod* pResultMethod = nullptr;

	_GetModuleInstanceMethod(strMethodName, this, &pResultMethod);

	return pResultMethod;
}

void IrisModule::_GetModuleInstanceMethod(const string & strMethodName, IrisModule * pCurModule, IrisMethod ** ppResultMethod)
{
	if (*ppResultMethod) {
		return;
	}

	IrisMethod* pMethod = nullptr;
	pMethod = static_cast<IrisObject*>(pCurModule->GetModuleObject())->GetInstanceMethod(strMethodName);

	if (pMethod) {
		*ppResultMethod = pMethod;
		return;
	}

	for (auto& module : pCurModule->GetModules()) {
		_GetModuleInstanceMethod(strMethodName, module.second, ppResultMethod);
		if (*ppResultMethod) {
			return;
		}
	}
}
