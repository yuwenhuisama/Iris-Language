#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include "IrisInterpreter/IrisStructure/IrisInterface.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisFatalErrorHandler.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"
#include "IrisUnil/IrisValues.h"

void IrisClass::_FunctionCollect(IrisInterface* pInterface, _InterfaceFunctionDeclareMap& mpFunctionDeclare) {
	// ����
	if (!pInterface)
		return;

	IrisInterface* pTmpInter = pInterface;

	// �Ӽ̳�����Ͷ˵Ľӿڿ�ʼ�������FunctionDeclare
	pTmpInter->m_iwlDecAddingLock.ReadLock();
	for (auto& funcdec : pTmpInter->m_mpFunctionDeclareMap){
		// û�ҵ�����ӽ�ȥ��Ĭ�ϼ̳�����Ͷ˵Ľӿڷ������帲�Ǹ��ӿڵķ������壩
		if (mpFunctionDeclare.find(funcdec.first) == mpFunctionDeclare.end()){
			mpFunctionDeclare.insert(_InterfaceFunctionDeclarePair(funcdec.first, funcdec.second));
		}
	}
	pTmpInter->m_iwlDecAddingLock.ReadUnlock();

	// ��һ��
	pTmpInter->m_iwlInfAddingLock.ReadLock();
	for (auto inter : pTmpInter->m_mpInterfaces){
		_FunctionCollect(inter, mpFunctionDeclare);
	}
	pTmpInter->m_iwlInfAddingLock.ReadUnlock();
}

bool IrisClass::_FunctionAchieved() {
	_InterfaceFunctionDeclareMap mpFunctionDeclare;
	// �ݹ���ã������еĽӿڵ�FunctionDeclare���ŵ�map����
	m_iwlInterfaceAddingWLLock.ReadLock();
	for (auto& inter : m_hsInterfaces){
		_FunctionCollect(inter, mpFunctionDeclare);
	}

	bool bResult = false;
	IrisMethod* pMethod = nullptr;
	// ��ʼ������
	for (auto& funcdec : mpFunctionDeclare) {
		pMethod = GetMethod(funcdec.first, bResult);
		// ���û���ҵ�����ôֱ���˳�
		if (!pMethod) {
			m_iwlInterfaceAddingWLLock.ReadUnlock();
			return false;
		}
		else {
			// ����ҵ��˵��ǲ������ԣ������˳�
			auto& pMethodDeclare = funcdec.second;
			if (pMethod->IsWithVariableParameter() != pMethodDeclare.m_bHaveVariableParameter
				|| pMethod->GetParameterAmount() != pMethodDeclare.m_nParameterAmount) {
				m_iwlInterfaceAddingWLLock.ReadUnlock();
				return false;
			}
		}
	}
	m_iwlInterfaceAddingWLLock.ReadUnlock();
	return true;
}

#if IR_USE_STL_STRING
void IrisClass::_SearchModuleConstance(SearchVariableType eType, const string& strVariableName, IrisModule* pCurModule, IrisValue** pValue) {
#else
void IrisClass::_SearchModuleConstance(SearchVariableType eType, const IrisInternString& strVariableName, IrisModule* pCurModule, IrisValue** pValue) {
#endif // IR_USE_STL_STRING
	//����
	if (pCurModule == nullptr) {
		return;
	}
	
	bool bResult = false;
	if (eType == SearchVariableType::Constance) {
		auto& ivResult = pCurModule->GetCurrentModuleConstance(strVariableName, bResult);
		if (bResult) {
			*pValue = (IrisValue*)&ivResult;
			return;
		}
	}
	else {
		auto& ivResult = pCurModule->GetCurrentModuleClassVariable(strVariableName, bResult);
		if (bResult) {
			*pValue = (IrisValue*)&ivResult;
			return;
		}
	}

	for (auto& module : pCurModule->GetModules()) {
		_SearchModuleConstance(eType, strVariableName, module, pValue);
		if (*pValue)
			return;
	}
}

#if IR_USE_STL_STRING
const IrisValue& IrisClass::SearchConstance(const string& strConstName, bool& bResult) {
#else
const IrisValue& IrisClass::SearchConstance(const IrisInternString& strConstName, bool& bResult) {
#endif // IR_USE_STL_STRING
	bResult = true;
	IrisValue* pValue = nullptr;
	// ����˳���ࡢ��������ģ�顢����
	// ����
	//decltype(m_hsConstances)::iterator iCons;
	//if ((iCons = m_hsConstances.find(strConstName)) != m_hsConstances.end()){
	//	return iCons->second;
	//}

	auto& ivResultValue = GetCurrentClassConstance(strConstName, bResult);
	if (bResult) {
		return ivResultValue;
	}

	bResult = true;
	// ��������ģ��
	IrisClass* pCurClass = this;
	do{
		for (auto& module : pCurClass->m_hsModules){
			_SearchModuleConstance(SearchVariableType::Constance, strConstName, module, &pValue);
			if (pValue){
				return *pValue;
			}
		}
		pCurClass = pCurClass->m_pSuperClass;
	} while (pCurClass);

	bResult = true;
	// ����
	pCurClass = m_pSuperClass;
	do{
		auto& ivResultValue = pCurClass->GetCurrentClassConstance(strConstName, bResult);
		if (bResult) {
			return ivResultValue;
		}
		pCurClass = pCurClass->m_pSuperClass;
	} while (pCurClass);

	bResult = true;
	pValue = (IrisValue*)&IrisInterpreter::CurrentInterpreter()->GetConstance(strConstName, bResult);
	if (bResult) {
		return *pValue;
	}

	bResult = false;
	return IrisDevUtil::Nil();
}

void IrisClass::AddInterface(IrisInterface* pInterface) {
	m_iwlInterfaceAddingWLLock.WriteLock();
	m_hsInterfaces.insert(pInterface);
	m_iwlInterfaceAddingWLLock.WriteUnlock();
}

void IrisClass::AddModule(IrisModule* pModule) {
	m_iwlModuleAddingWLLock.WriteLock();
	m_hsModules.insert(pModule);
	m_iwlModuleAddingWLLock.WriteUnlock();
}

#if IR_USE_STL_STRING
const IrisValue& IrisClass::SearchClassVariable(const string& strClassVariableName, bool& bResult) {
#else
const IrisValue& IrisClass::SearchClassVariable(const IrisInternString& strClassVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
	bResult = true;
	IrisValue* pValue = nullptr;
	// ����˳���ࡢ��������ģ�顢����
	// ����
	auto& ivResultValue = GetCurrentClassConstance(strClassVariableName, bResult);
	if (bResult) {
		return ivResultValue;
	}

	bResult = true;
	// ��������ģ��
	IrisClass* pCurClass = this;
	do {
		for (auto& module : pCurClass->m_hsModules) {
			_SearchModuleConstance(SearchVariableType::ClassInstance, strClassVariableName, module, &pValue);
			if (pValue) {
				return *pValue;
			}
		}
		pCurClass = pCurClass->m_pSuperClass;
	} while (pCurClass);

	bResult = true;
	// ����
	pCurClass = m_pSuperClass;
	do {
		auto& ivResultValue = pCurClass->GetCurrentClassClassVariable(strClassVariableName, bResult);
		if (bResult) {
			return ivResultValue;
		}
		pCurClass = pCurClass->m_pSuperClass;
	} while (pCurClass);

	bResult = false;
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisClass::CreateInstance(IIrisValues* ivsParams, IIrisContextEnvironment* pContexEnvironment) {
	// ����Ƿ���û��ʵ�ֵĽӿ�
	auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
	IrisValue ivValue;
	if (!m_bIsCompleteClass){
		if (!(m_bIsCompleteClass = _FunctionAchieved())) {
			// **Error**
#if IR_USE_STL_STRING
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ClassNotCompleteIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Class " + m_strClassName + " is still having some methods not defined but declared in the interfaces jointed.");
#else
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ClassNotCompleteIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Class " + m_strClassName.GetSTLString() + " is still having some methods not defined but declared in the interfaces jointed.");
#endif // IR_USE_STL_STRING

			return IrisInterpreter::CurrentInterpreter()->Nil();
		}
	}
	// ���ɶ���
	IrisObject* pObject = new IrisObject();
	pObject->SetClass(m_pExternClass);
	if (IrisDevUtil::GetCurrentThreadInfo()->m_nNativeReference > 0) {
		//pObject->SetIsCreateInNativeFunction(true);
		IrisDevUtil::GetCurrentThreadInfo()->m_hpObjectInNativeFunctionHeap.insert(pObject);
	}
	if (IsObjectClass()) {
		pObject->SetNativeObject(m_pExternClass->NativeAlloc());
		IrisGC::CurrentGC()->AddSize(sizeof(IrisObject));
	}
	else {
		pObject->SetNativeObject(m_pExternClass->NativeAlloc());
		IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
	}
	IrisGC::CurrentGC()->Start();
	ivValue.SetIrisObject(pObject);
	if(IsNormalClass()){

		IrisDevUtil::CallMethod(IrisValue::WrapObjectPointerToIrisValue(pObject), "__format",  ivsParams, pContexEnvironment);

		//IrisDevUtil::GetCurrentThreadInfo()->m_skTempNewObjectStack.push_back(pObject);

		//if (ivsParams) {
		//	auto& vcVictor = static_cast<IrisValues*>(ivsParams)->GetVector();
		//	auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
		//	for (auto& value : vcVictor) {
		//		pInfo->m_stStack.Push(value);
		//	}
		//}

		//pObject->CallInstanceFunction("__format", pContexEnvironment, static_cast<IrisValues*>(ivsParams), CallerSide::Outside);

		//if (ivsParams && !IrisDevUtil::IrregularHappened()) {
		//	IrisInterpreter::CurrentInterpreter()->PopStack(static_cast<IrisValues*>(ivsParams)->GetSize());
		//}
		//IrisDevUtil::GetCurrentThreadInfo()->m_skTempNewObjectStack.pop_back();
	}

	// ���¶��󱣴浽����
	IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);

	return ivValue;
}

#if IR_USE_STL_STRING
void IrisClass::_ClassModuleMethodSearch(IrisClass* pCurClass, const string& strMethodName, IrisMethod** ppMethod) {
#else
void IrisClass::_ClassModuleMethodSearch(IrisClass* pCurClass, const IrisInternString& strMethodName, IrisMethod** ppMethod) {
#endif // IR_USE_STL_STRING
	for (auto& module : pCurClass->m_hsModules) {
		_ModuleMethodSearch(strMethodName, module, ppMethod);
		// �ҵ����˳�
		if (*ppMethod)
			return;
	}
}

#if IR_USE_STL_STRING
void IrisClass::_ModuleMethodSearch(const string& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {
#else
void IrisClass::_ModuleMethodSearch(const IrisInternString& strFunctionName, IrisModule* pCurModule, IrisMethod** ppMethod) {
#endif // IR_USE_STL_STRING
	// ����
	if (!pCurModule)
		return;
	// ����ҵ�
	// �෽��
	IrisMethod* pMethod = nullptr;
	//if (eSearchType == SearchMethodType::ClassMethod) {
		//pMethod = pCurModule->GetCurrentModuleMethod(IrisModule::SearchMethodType::ClassMethod, strFunctionName);
	//}
	// ʵ������
	//else {
		pMethod = pCurModule->GetCurrentModuleMethod(strFunctionName);
	//}
	if (pMethod) {
		*ppMethod = pMethod;
		return;
	}
	// �����ݹ����
	for (auto& module : pCurModule->GetModules()) {
		_ModuleMethodSearch(strFunctionName, module, ppMethod);
		if (*ppMethod)
			break;
	}
}

#if IR_USE_STL_STRING
IrisMethod* IrisClass::GetMethod(const string& strMethodName, bool& bIsCurClassMethod) {
#else
IrisMethod* IrisClass::GetMethod(const IrisInternString& strMethodName, bool& bIsCurClassMethod) {
#endif // IR_USE_STL_STRING
	// �����Ĳ���˳�򣺱��ࡢ�������������ģ�飬������Ҳ����Ǿ����Ҳ�����
	// ��鱾��
	// ����ǵ����෽���������
	bIsCurClassMethod = false;
	IrisMethod* pResultMethod = nullptr;
	if (pResultMethod = GetCurrentClassMethod(strMethodName)) {
		bIsCurClassMethod = true;
		return pResultMethod;
	}
	else {
		// ��������������ģ��
		IrisClass* pCurClass = this;

		// ������������ģ��
		_ClassModuleMethodSearch(this, strMethodName, &pResultMethod);
		if (pResultMethod) {
			bIsCurClassMethod = true;
			return pResultMethod;
		}

		pCurClass = pCurClass->m_pSuperClass;
		// ����ɨ��̳���
		// �Լ̳�������������������ģ�����ɨ��
		while (pCurClass) {
			// ����
			pResultMethod = pCurClass->GetCurrentClassMethod(strMethodName);
			if (pResultMethod) {
				// �ҵ�����
				bIsCurClassMethod = false;
				return pResultMethod;
			}
			// ���������ģ��
			_ClassModuleMethodSearch(pCurClass, strMethodName, &pResultMethod);
			if (pResultMethod) {
				// �ҵ�����
				bIsCurClassMethod = false;
				return pResultMethod;
			}
			pCurClass = pCurClass->m_pSuperClass;
		}
	}

	// ����֮�������Ҳ�����
	return nullptr;
}

#if IR_USE_STL_STRING
IrisClass::IrisClass(const string& strClassName, IrisClass* pSuperClass, IrisModule* pUpperModule, IIrisClass* pExternClass) : m_strClassName(strClassName), m_pSuperClass(pSuperClass), m_pUpperModule(pUpperModule), m_pExternClass(pExternClass) {
#else
IrisClass::IrisClass(const IrisInternString& strClassName, IrisClass* pSuperClass, IrisModule* pUpperModule, IIrisClass* pExternClass) : m_strClassName(strClassName), m_pSuperClass(pSuperClass), m_pUpperModule(pUpperModule), m_pExternClass(pExternClass) {
#endif // IR_USE_STL_STRING
	if (IrisInterpreter::CurrentInterpreter()->GetIrisClass("Class")) {
		IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Class")->CreateInstance(nullptr, nullptr);
		IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivValue)->SetClass(this);
		m_pClassObject = ivValue.GetIrisObject();
	}
	else {
		IrisValue ivValue;
		IrisObject* pObject = new IrisObject();
		pObject->SetClass(m_pExternClass);
		pObject->SetNativeObject(m_pExternClass->NativeAlloc());

		IrisGC::CurrentGC()->AddSize(sizeof(IrisObject) + pObject->GetClass()->GetTrustteeSize(pObject->GetNativeObject()));
		IrisGC::CurrentGC()->Start();
		ivValue.SetIrisObject(pObject);

		static_cast<IrisClassBaseTag*>(pObject->GetNativeObject())->SetClass(this);
		m_pClassObject = pObject;
		

		// ���¶��󱣴浽����
		IrisInterpreter::CurrentInterpreter()->AddNewInstanceToHeap(ivValue);

	}
}

#if IR_USE_STL_STRING
IrisValue IrisClass::CallClassMethod(const string& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide) {
#else
IrisValue IrisClass::CallClassMethod(const IrisInternString& strMethodName, IrisContextEnvironment* pContextEnvironment, IrisValues* ivParameters, CallerSide eSide) {
#endif // IR_USE_STL_STRING
	return static_cast<IrisObject*>(m_pClassObject)->CallInstanceFunction(strMethodName, pContextEnvironment, ivParameters, eSide);
}

void IrisClass::ResetAllMethodsObject() {
	auto& hsInsMethods = static_cast<IrisObject*>(m_pClassObject)->m_mpInstanceMethods;

	for (auto& method : hsInsMethods) {
		method.second->ResetObject();
	}

	for (auto& instancemethod : m_hsInstanceMethods) {
		instancemethod.second->ResetObject();
	}
}

#if IR_USE_STL_STRING
IrisMethod* IrisClass::GetCurrentClassMethod(const string& strMethodName) {
#else
IrisMethod* IrisClass::GetCurrentClassMethod(const IrisInternString& strMethodName) {
#endif // IR_USE_STL_STRING
	IrisMethod* pResultMethod = nullptr;
	m_iwlInstanceMethodWLLock.ReadLock();
	decltype(m_hsInstanceMethods)::iterator iMethod;
	if ((iMethod = m_hsInstanceMethods.find(strMethodName)) == m_hsInstanceMethods.end()) {
		pResultMethod = nullptr;
	}
	else {
		pResultMethod = iMethod->second;
	}
	m_iwlInstanceMethodWLLock.ReadUnlock();
	return pResultMethod;
}

#if IR_USE_STL_STRING
const IrisValue & IrisClass::GetCurrentClassClassVariable(const string & strVariableName, bool & bResult)
#else
const IrisValue & IrisClass::GetCurrentClassClassVariable(const IrisInternString & strVariableName, bool & bResult)
#endif // IR_USE_STL_STRING
{
	decltype(m_hsClassVariables)::iterator iVariable;
	m_iwlClassClassVariableWLLock.ReadLock();
	if ((iVariable = m_hsClassVariables.find(strVariableName)) != m_hsClassVariables.end()) {
		auto& ivResult = iVariable->second;
		bResult = true;
		m_iwlClassClassVariableWLLock.ReadUnlock();
		return ivResult;
	}
	else {
		bResult = false;
		m_iwlClassClassVariableWLLock.ReadUnlock();
		return IrisDevUtil::Nil();
	}
}

#if IR_USE_STL_STRING
const IrisValue & IrisClass::GetCurrentClassConstance(const string & strConstanceName, bool & bResult)
#else
const IrisValue & IrisClass::GetCurrentClassConstance(const IrisInternString & strConstanceName, bool & bResult)
#endif // IR_USE_STL_STRING
{
	decltype(m_hsConstances)::iterator iVariable;
	m_iwlClassConstanceWLLock.ReadLock();
	if ((iVariable = m_hsConstances.find(strConstanceName)) != m_hsConstances.end()) {
		auto& ivResult = iVariable->second;
		m_iwlClassConstanceWLLock.ReadUnlock();
		bResult = true;
		return ivResult;
	}
	else {
		bResult = false;
		m_iwlClassConstanceWLLock.ReadUnlock();
		return IrisDevUtil::Nil();
	}
}

#if IR_USE_STL_STRING
void IrisClass::AddSetter(const string& strProperName, IrisNativeFunction pfMethod) {
	string strMethodName = strProperName;
#else
void IrisClass::AddSetter(const IrisInternString& strProperName, IrisNativeFunction pfMethod) {
	string strMethodName = strProperName.GetSTLString();
#endif // IR_USE_STL_STRING
	strMethodName.assign(strMethodName.begin() + 1, strMethodName.end());
	strMethodName = "__set_" + strMethodName;
	IrisMethod* pMethod = new IrisMethod(strMethodName, pfMethod, 1, false);
	pMethod->SetMethodName(strMethodName);

	m_iwlInstanceMethodWLLock.WriteLock();
	decltype(m_hsInstanceMethods)::iterator iMethod;
	if ((iMethod = m_hsInstanceMethods.find(pMethod->GetMethodName())) != m_hsInstanceMethods.end()) {
		delete iMethod->second;
		m_hsInstanceMethods[pMethod->GetMethodName()] = pMethod;
	}
	else {
		m_hsInstanceMethods.insert(_MethodPair(pMethod->GetMethodName(), pMethod));
	}
	m_iwlInstanceMethodWLLock.WriteUnlock();
}

#if IR_USE_STL_STRING
void IrisClass::AddGetter(const string& strProperName, IrisNativeFunction pfMethod) {
	string strMethodName = strProperName;
#else
void IrisClass::AddGetter(const IrisInternString& strProperName, IrisNativeFunction pfMethod) {
	string strMethodName = strProperName.GetSTLString();
#endif // IR_USE_STL_STRING
	strMethodName.assign(strMethodName.begin() + 1, strMethodName.end());
	strMethodName = "__get_" + strMethodName;
	IrisMethod* pMethod = new IrisMethod(strMethodName, pfMethod, 0, false);
	pMethod->SetMethodName(strMethodName);

	m_iwlInstanceMethodWLLock.WriteLock();
	decltype(m_hsInstanceMethods)::iterator iMethod;
	if ((iMethod = m_hsInstanceMethods.find(pMethod->GetMethodName())) != m_hsInstanceMethods.end()) {
		delete iMethod->second;
		m_hsInstanceMethods[pMethod->GetMethodName()] = pMethod;
	}
	else {
		m_hsInstanceMethods.insert(_MethodPair(pMethod->GetMethodName(), pMethod));
	}
	m_iwlInstanceMethodWLLock.WriteUnlock();
}

void IrisClass::ResetNativeObject() {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Class")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivValue)->SetClass(this);
	m_pClassObject = static_cast<IrisObject*>(ivValue.GetIrisObject());
}


 void IrisClass::ClearInvolvingModules() {
	m_iwlModuleAddingWLLock.WriteLock();
	m_hsModules.clear();
	m_iwlModuleAddingWLLock.WriteUnlock();
}

 void IrisClass::ClearJointingInterfaces() {
	m_iwlInterfaceAddingWLLock.WriteLock();
	m_hsInterfaces.clear();
	m_iwlInterfaceAddingWLLock.WriteUnlock();
}


#if IR_USE_STL_STRING
 void IrisClass::AddConstance(const string& strConstName, const IrisValue& ivInitialValue) {
#else
 void IrisClass::AddConstance(const IrisInternString& strConstName, const IrisValue& ivInitialValue) {
#endif // IR_USE_STL_STRING
	m_iwlClassConstanceWLLock.WriteLock();
	m_hsConstances.insert(_VariablePair(strConstName, ivInitialValue));
	m_iwlClassConstanceWLLock.WriteUnlock();
}

 void IrisClass::AddClassMethod(IrisMethod* pMethod) {
	 static_cast<IrisObject*>(m_pClassObject)->AddSingleInstanceMethod(pMethod);
}

 void IrisClass::AddInstanceMethod(IrisMethod* pMethod) {
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
 void IrisClass::AddClassVariable(const string& strClassVariableName) {
#else
 void IrisClass::AddClassVariable(const IrisInternString& strClassVariableName) {
#endif // IR_USE_STL_STRING
	m_iwlClassClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, IrisInterpreter::CurrentInterpreter()->Nil()));
	m_iwlClassClassVariableWLLock.WriteUnlock();
}
#if IR_USE_STL_STRING
 void IrisClass::AddClassVariable(const string& strClassVariableName, const IrisValue& ivInitialValue) {
#else
 void IrisClass::AddClassVariable(const IrisInternString& strClassVariableName, const IrisValue& ivInitialValue) {
#endif // IR_USE_STL_STRING
    m_iwlClassClassVariableWLLock.WriteLock();
	m_hsClassVariables.insert(_VariablePair(strClassVariableName, ivInitialValue));
	m_iwlClassClassVariableWLLock.WriteUnlock();
}

IrisClass::~IrisClass() {
	for (auto iter : m_hsInstanceMethods){
		delete iter.second;
	}
	//for (auto iter : m_hsClassMethods){
	//	delete iter.second;
	//}
	delete m_pExternClass;
}