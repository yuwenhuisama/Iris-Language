#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisFatalErrorHandler.h"


void IrisModule::_SearchConstance(const IrisInternString& strConstanceName, IrisModule* pCurModule, IrisValue** pValue) {
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

	for (auto& module : pCurModule->m_hsModules) {
		_SearchConstance(strConstanceName, module, pValue);
		if (pValue)
			return;
	}
}
void IrisModule::_SearchClassVariable(const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue){
	//出口
	if (pCurModule == nullptr) {
		return;
	}

	bool bResult = false;
	auto& ivResultValue = pCurModule->GetCurrentModuleClassVariable(strVariableName, bResult);
	if (bResult) {
		*pValue = (IrisValue*)&ivResultValue;
	}

	for (auto& module : pCurModule->m_hsModules) {
		_SearchClassVariable(strVariableName, module, pValue);
		if (*pValue)
			return;
	}
}

const IrisValue& IrisModule::SearchConstance(const IrisInternString& strConstName, bool& bResult) {
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
	m_iwlModuleAddingWLLock.WriteLock();
	m_hsModules.insert(pModule);
	m_iwlModuleAddingWLLock.WriteUnlock();
}

void IrisModule::AddInvolvingModule(IrisModule * pModule)
{
	m_iwlModuleInvolvingWLLock.WriteLock();
	if (m_hsInvolveModules.find(pModule->GetModuleName()) == m_hsInvolveModules.end()) {
		m_hsInvolveModules.insert(_ModulePair(pModule->GetModuleName(), pModule));
	}
	m_iwlModuleInvolvingWLLock.WriteUnlock();
}

const IrisValue& IrisModule::SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult) {
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

void IrisModule::_SearchModuleMethod(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {

	// 出口
	if (*ppMethod)
		return;

	IrisMethod* pMethod = nullptr;
	pMethod = pCurModule->GetCurrentModuleMethod(strFunctionName);
	if (pMethod) {
		*ppMethod = pMethod;
		return;
	}

	for (auto& module : pCurModule->m_hsModules) {
		_SearchModuleMethod(strFunctionName, module, ppMethod);
		if (*ppMethod)
			return;
	}
}

IrisValue IrisModule::CallClassMethod(const IrisInternString& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide, unsigned int nLineNumber, int nBelongingFileIndex) {
	return static_cast<IrisObject*>(m_pModuleObject)->CallInstanceFunction(strMethodName, pContextEnvironment, ivParameters, eSide, nLineNumber, nBelongingFileIndex);
}

IrisModule::IrisModule(const IrisInternString& strModuleName, IrisModule* pUpperModule) : m_strModuleName(strModuleName), m_pUpperModule(pUpperModule) {
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

	for (auto& instancemethod : m_hsInstanceMethods) {
		instancemethod.second->ResetObject();
	}
}

IrisMethod* IrisModule::GetCurrentModuleMethod(const IrisInternString& strMethodName) {
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

const IrisValue & IrisModule::GetCurrentModuleClassVariable(const IrisInternString& strVariableName, bool& bResult)
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

const IrisValue & IrisModule::GetCurrentModuleConstance(const IrisInternString& strConstanceName, bool& bResult)
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

void IrisModule::AddConstance(const IrisInternString & strConstName, const IrisValue & ivInitialValue) {
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
void IrisModule::AddClassVariable(const IrisInternString & strClassVariableName) {
	m_iwlModuleClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, IrisInterpreter::CurrentInterpreter()->Nil()));
	m_iwlModuleClassVariableWLLock.WriteUnlock();
}

void IrisModule::AddClassVariable(const IrisInternString & strClassVariableName, const IrisValue & ivInitialValue) {
	m_iwlModuleClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, ivInitialValue));
	m_iwlModuleClassVariableWLLock.WriteUnlock();
}

void IrisModule::AddClass(IrisClass * pClass) {
	//decltype(m_hsClasses)::iterator iClass;
	m_iwlClassAddingWLLock.WriteLock();
	m_hsClasses.insert(pClass);
	m_iwlClassAddingWLLock.WriteUnlock();
}

void IrisModule::AddInvolvedInterface(IrisInterface * pInterface) {
	m_iwlInterfaceAddingWLLock.WriteLock();
	decltype(m_hsClasses)::iterator iInterface;
	m_hsInvolvedInterfaces.insert(pInterface);
	m_iwlInterfaceAddingWLLock.WriteUnlock();
}

IrisClass * IrisModule::GetClass(const IrisInternString & strClassName) {
	IrisClass* pClass = nullptr;
	m_iwlClassAddingWLLock.ReadLock();
	if (m_hsConstances.find(strClassName) != m_hsConstances.end()) {
		auto& ivConst = m_hsConstances[strClassName];
		if (IrisDevUtil::CheckClassIsClass(ivConst)) {
			pClass = IrisDevUtil::GetNativeClass(ivConst)->GetInternClass();
		}
	}
	m_iwlClassAddingWLLock.ReadUnlock();
	return pClass;
}

IrisInterface * IrisModule::GetInterface(const IrisInternString & strInterfaceName) {
	IrisInterface* pInterface = nullptr;
	m_iwlInterfaceAddingWLLock.ReadLock();
	if (m_hsConstances.find(strInterfaceName) != m_hsConstances.end()) {
		auto& ivConst = m_hsConstances[strInterfaceName];
		if (IrisDevUtil::CheckClassIsClass(ivConst)) {
			pInterface = IrisDevUtil::GetNativeInterface(ivConst)->GetInternInterface();
		}
	}
	m_iwlInterfaceAddingWLLock.ReadUnlock();
	return pInterface;
}

IrisMethod * IrisModule::GetModuleInstanceMethod(const IrisInternString& strMethodName)
{
	IrisMethod* pResultMethod = nullptr;

	_GetModuleInstanceMethod(strMethodName, this, &pResultMethod);

	return pResultMethod;
}

void IrisModule::_GetModuleInstanceMethod(const IrisInternString & strMethodName, IrisModule * pCurModule, IrisMethod ** ppResultMethod)
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
		_GetModuleInstanceMethod(strMethodName, module, ppResultMethod);
		if (*ppResultMethod) {
			return;
		}
	}
}
