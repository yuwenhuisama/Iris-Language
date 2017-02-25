#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisFatalErrorHandler.h"


#if IR_USE_STL_STRING
void IrisModule::_SearchConstance(const string& strConstanceName, IrisModule* pCurModule, IrisValue** pValue) {
#else
void IrisModule::_SearchConstance(const IrisInternString& strConstanceName, IrisModule* pCurModule, IrisValue** pValue) {
#endif // IR_USE_STL_STRING
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
#if IR_USE_STL_STRING
void IrisModule::_SearchClassVariable(const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue) {
#else
void IrisModule::_SearchClassVariable(const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue) {
#endif // IR_USE_STL_STRING
	//出口
	if (pCurModule == nullptr) {
		return;
	}

	bool bResult = false;
	auto& ivResultValue = pCurModule->GetCurrentModuleClassVariable(strVariableName, bResult);
	if (bResult) {
		*pValue = const_cast<IrisValue*>(&ivResultValue);
	}

	for (auto& module : pCurModule->m_hsModules) {
		_SearchClassVariable(strVariableName, module, pValue);
		if (*pValue)
			return;
	}
}

#if IR_USE_STL_STRING
const IrisValue& IrisModule::SearchConstance(const string& strConstName, bool& bResult) {
#else
const IrisValue& IrisModule::SearchConstance(const IrisInternString& strConstName, bool& bResult) {
#endif // IR_USE_STL_STRING
	bResult = true;
	IrisValue* pValue = nullptr;

	_SearchConstance(strConstName, this, &pValue);
	if (pValue){
		return *pValue;
	}
	else{
		pValue = const_cast<IrisValue*>(&IrisInterpreter::CurrentInterpreter()->GetOtherValue(strConstName, bResult));
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

#if IR_USE_STL_STRING
const IrisValue& IrisModule::SearchClassVariable(const string& strClassVariableName, bool& bResult) {
#else
const IrisValue& IrisModule::SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
void IrisModule::_SearchModuleMethod(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {
#else
void IrisModule::_SearchModuleMethod(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {
#endif // IR_USE_STL_STRING

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

#if IR_USE_STL_STRING
IrisValue IrisModule::CallClassMethod(const string& strMethodName, IrisValues* ivParameters, IrisContextEnvironment* pContextEnvironment, IrisThreadInfo* pThreadInfo, CallerSide eSide) {
#else
IrisValue IrisModule::CallClassMethod(const IrisInternString& strMethodName, IrisValues* ivParameters, IrisContextEnvironment* pContextEnvironment, IrisThreadInfo* pThreadInfo, CallerSide eSide) {
#endif // IR_USE_STL_STRING
	return static_cast<IrisObject*>(m_pModuleObject)->CallInstanceFunction(strMethodName, ivParameters, pContextEnvironment, pThreadInfo, eSide);
}

#if IR_USE_STL_STRING
IrisModule::IrisModule(const string& strModuleName, IrisModule* pUpperModule) : m_strModuleName(strModuleName), m_pUpperModule(pUpperModule) {
#else
IrisModule::IrisModule(const IrisInternString& strModuleName, IrisModule* pUpperModule) : m_strModuleName(strModuleName), m_pUpperModule(pUpperModule) {
#endif // IR_USE_STL_STRING
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Module")->CreateInstance(nullptr, nullptr, IrisThreadManager::CurrentThreadManager()->GetThreadInfo(this_thread::get_id()));
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

#if IR_USE_STL_STRING
IrisMethod* IrisModule::GetCurrentModuleMethod(const string& strMethodName) {
#else
IrisMethod* IrisModule::GetCurrentModuleMethod(const IrisInternString& strMethodName) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
const IrisValue & IrisModule::GetCurrentModuleClassVariable(const string& strVariableName, bool& bResult)
#else
const IrisValue & IrisModule::GetCurrentModuleClassVariable(const IrisInternString& strVariableName, bool& bResult)
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
const IrisValue & IrisModule::GetCurrentModuleConstance(const string& strConstanceName, bool& bResult)
#else
const IrisValue & IrisModule::GetCurrentModuleConstance(const IrisInternString& strConstanceName, bool& bResult)
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
void IrisModule::AddConstance(const string & strConstName, const IrisValue & ivInitialValue) {
#else
void IrisModule::AddConstance(const IrisInternString & strConstName, const IrisValue & ivInitialValue) {
#endif // IR_USE_STL_STRING
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
		//delete iMethod->second;
		m_hsInstanceMethods[pMethod->GetMethodName()] = pMethod;
	}
	else {
		m_hsInstanceMethods.insert(_MethodPair(pMethod->GetMethodName(), pMethod));
	}
	m_iwlInstanceMethodWLLock.WriteUnlock();
}

#if IR_USE_STL_STRING
void IrisModule::AddClassVariable(const string & strClassVariableName) {
#else
void IrisModule::AddClassVariable(const IrisInternString & strClassVariableName) {
#endif // IR_USE_STL_STRING
	m_iwlModuleClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, IrisInterpreter::CurrentInterpreter()->Nil()));
	m_iwlModuleClassVariableWLLock.WriteUnlock();
}

#if IR_USE_STL_STRING
void IrisModule::AddClassVariable(const string & strClassVariableName, const IrisValue & ivInitialValue) {
#else
void IrisModule::AddClassVariable(const IrisInternString & strClassVariableName, const IrisValue & ivInitialValue) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
IrisClass * IrisModule::GetClass(const string & strClassName) {
#else
IrisClass * IrisModule::GetClass(const IrisInternString & strClassName) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
IrisInterface * IrisModule::GetInterface(const string & strInterfaceName) {
#else
IrisInterface * IrisModule::GetInterface(const IrisInternString & strInterfaceName) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
IrisMethod * IrisModule::GetModuleInstanceMethod(const string& strMethodName)
#else
IrisMethod * IrisModule::GetModuleInstanceMethod(const IrisInternString& strMethodName)
#endif // IR_USE_STL_STRING
{
	IrisMethod* pResultMethod = nullptr;

	_GetModuleInstanceMethod(strMethodName, this, &pResultMethod);

	return pResultMethod;
}

#if IR_USE_STL_STRING
void IrisModule::_GetModuleInstanceMethod(const string & strMethodName, IrisModule * pCurModule, IrisMethod ** ppResultMethod)
#else
void IrisModule::_GetModuleInstanceMethod(const IrisInternString & strMethodName, IrisModule * pCurModule, IrisMethod ** ppResultMethod)
#endif // IR_USE_STL_STRING
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
