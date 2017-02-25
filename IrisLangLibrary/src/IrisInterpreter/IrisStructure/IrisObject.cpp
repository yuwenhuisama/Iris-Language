#include "IrisInterpreter/IrisStructure/IrisObject.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisUnil/IrisValues.h"
#include "IrisFatalErrorHandler.h"

#include "IrisThread/IrisThreadInfo.h"

unsigned int IrisObject::s_nMaxID = 0;

void IrisObject::Mark() {
	m_bIsMaked = true;
	m_pClass->Mark(m_pNativeObject);
	for (auto value : m_mpInstanceMethods) {
		static_cast<IrisObject*>(value.second->GetMethodObject())->Mark();
	}
	for (auto value : m_mpInstanceVariables) {
		static_cast<IrisObject*>(value.second.GetIrisObject())->Mark();
	}
}

IrisObject::IrisObject() {
	m_nObjectID = ++s_nMaxID;
}

#if IR_USE_STL_STRING
IrisValue IrisObject::CallInstanceFunction(const string& strFunctionName, IIrisValues* ivsValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo, CallerSide eSide) {
#else
IrisValue IrisObject::CallInstanceFunction(const IrisInternString& strFunctionName, IIrisValues* ivsValues, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo, CallerSide eSide) {
#endif // IR_USE_STL_STRING
	// �����Լ���Instance Functions��Ѱ�Ҷ�Ӧ����

	auto pInfo = static_cast<IrisThreadInfo*>(pThreadInfo);

	IrisMethod* pMethod = nullptr;
	bool bIsCurClassMethod = false;

	auto iMethod = m_mpInstanceMethods.find(strFunctionName);
	if (iMethod != m_mpInstanceMethods.end()){
		pMethod = iMethod->second;
		bIsCurClassMethod = true;
	} else {
		pMethod = m_pClass->GetInternClass()->GetMethod(strFunctionName, bIsCurClassMethod);
	}

	IrisValue ivResult;
	if (pMethod) {
		IrisValue ivValue;
		ivValue.SetIrisObject(this);

		// �ڲ�����
		if (eSide == CallerSide::Inside) {
			// �����ڲ�����������
			if (bIsCurClassMethod) {
				ivResult = pMethod->Call(ivValue, static_cast<IrisValues*>(ivsValues), static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisThreadInfo*>(pThreadInfo));
			}
			// ������ø��෽��
			else {
				// ��ֹ����
				if (pMethod->GetAuthority() == IrisMethod::MethodAuthority::Personal) {
					// **Error**
#if IR_USE_STL_STRING
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " is PERSONAL and cannot be called from derrived class " + m_pClass->GetInternClass()->GetClassName() + ".", pInfo);
#else
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " is PERSONAL and cannot be called from derrived class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + ".", pInfo);
#endif // IR_USE_STL_STRING
					ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
				}
				else {
					ivResult = pMethod->Call(ivValue, static_cast<IrisValues*>(ivsValues), static_cast<IrisContextEnvironment*>(pContextEnvironment), pInfo);
				}
			}
		}
		// �ⲿ����
		else {
			// ֻ�ܵ���EveryOne�ķ���
			// ��ֹ����
			if (pMethod->GetAuthority() != IrisMethod::MethodAuthority::Everyone) {
				// **Error**
#if IR_USE_STL_STRING
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " is not EVERYONE and cannot be called outside the class " + m_pClass->GetInternClass()->GetClassName() + " .", pInfo);
#else
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " is not EVERYONE and cannot be called outside the class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + " .", pInfo);
#endif // IR_USE_STL_STRING
				ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
			}
			else {
				ivResult = pMethod->Call(ivValue, static_cast<IrisValues*>(ivsValues), static_cast<IrisContextEnvironment*>(pContextEnvironment), pInfo);
			}
		}
	}
	else {
		// **Error**
#if IR_USE_STL_STRING
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " not found in class " + m_pClass->GetInternClass()->GetClassName() + ".", pInfo);
#else
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " not found in class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + ".", pInfo);
#endif // IR_USE_STL_STRING
		ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
	}

	return ivResult;
}

#if IR_USE_STL_STRING
const IrisValue & IrisObject::GetInstanceValue(const string & strInstanceValueName, bool & bResult) {
#else
const IrisValue & IrisObject::GetInstanceValue(const IrisInternString & strInstanceValueName, bool & bResult) {
#endif // IR_USE_STL_STRING
	m_iwlInstanceValueWLLock.ReadLock();
	decltype(m_mpInstanceVariables)::iterator iVariable;
	if ((iVariable = m_mpInstanceVariables.find(strInstanceValueName)) == m_mpInstanceVariables.end()) {
		bResult = false;
		m_iwlInstanceValueWLLock.ReadUnlock();
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
	bResult = true;
	auto& ivResult = iVariable->second;
	m_iwlInstanceValueWLLock.ReadUnlock();
	return ivResult;
}

#if IR_USE_STL_STRING
IrisMethod * IrisObject::GetInstanceMethod(const string & strInstanceMethodName)
#else
IrisMethod * IrisObject::GetInstanceMethod(const IrisInternString & strInstanceMethodName)
#endif // IR_USE_STL_STRING
{
	m_iwlInstanceMethodWLLock.ReadLock();
	decltype(m_mpInstanceMethods)::iterator iVariable = m_mpInstanceMethods.find(strInstanceMethodName);
	if (iVariable == m_mpInstanceMethods.end()) {
		m_iwlInstanceMethodWLLock.ReadUnlock();
		return nullptr;
	}
	m_iwlInstanceMethodWLLock.ReadUnlock();
	return iVariable->second;
}

#if IR_USE_STL_STRING
void IrisObject::AddInstanceValue(const string & strInstanceValueName, const IrisValue & ivValue) {
#else
void IrisObject::AddInstanceValue(const IrisInternString & strInstanceValueName, const IrisValue & ivValue) {
#endif // IR_USE_STL_STRING
	m_iwlInstanceValueWLLock.WriteLock();
	m_mpInstanceVariables.insert(_VariablePair(strInstanceValueName, ivValue));
	m_iwlInstanceValueWLLock.WriteUnlock();
}

void IrisObject::AddSingleInstanceMethod(IrisMethod * pMethod)
{
	m_iwlInstanceMethodWLLock.WriteLock();
	decltype(m_mpInstanceMethods)::iterator iFind = m_mpInstanceMethods.find(pMethod->GetMethodName());

	if (iFind == m_mpInstanceMethods.end()) {
		m_mpInstanceMethods.insert(_MethodPair(pMethod->GetMethodName(), pMethod));
	}
	else {
		m_mpInstanceMethods[pMethod->GetMethodName()] = pMethod;
	}

	m_iwlInstanceMethodWLLock.WriteUnlock();
}

IrisObject::~IrisObject() {
	this->m_pClass->NativeFree(m_pNativeObject);

	for (auto iter : m_mpInstanceMethods) {
		delete iter.second;
	}
}