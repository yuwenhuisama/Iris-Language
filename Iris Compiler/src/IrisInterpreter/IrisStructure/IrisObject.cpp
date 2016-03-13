#include "IrisInterpreter/IrisStructure/IrisObject.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisUnil/IrisValues.h"
#include "IrisFatalErrorHandler.h"

unsigned int IrisObject::s_nMaxID = 0;

void IrisObject::Mark() {
	m_bIsMaked = true;
	m_pClass->Mark(m_pNativeObject);
	for (auto value : m_mpInstanceMethods) {
		value.second->GetMethodObject()->Mark();
	}
	for (auto value : m_mpInstanceVariables) {
		value.second.GetIrisObject()->Mark();
	}
}

IrisObject::IrisObject() {
	m_nObjectID = ++s_nMaxID;
}

IrisValue IrisObject::CallInstanceFunction(const string& strFunctionName, IIrisContextEnvironment* pContextEnvironment, IIrisValues* ivsValues, CallerSide eSide, unsigned int nLineNumber, int nBelongingFileIndex) {

	// 先在自己的Instance Functions中寻找对应方法
	IrisMethod* pMethod = nullptr;
	bool bIsCurClassMethod = false;
	decltype(m_mpInstanceMethods)::iterator iMethod = m_mpInstanceMethods.find(strFunctionName);
	if (!(iMethod == m_mpInstanceMethods.end()))
		pMethod = iMethod->second;
	else {
		pMethod = m_pClass->GetInternClass()->GetMethod(IrisClass::SearchMethodType::InstanceMethod, strFunctionName, bIsCurClassMethod);
	}

	IrisValue ivResult;
	if (pMethod) {
		IrisValue ivValue;
		ivValue.SetIrisObject(this);

		// 内部调用
		if (eSide == CallerSide::Inside) {
			// 本类内部调用无限制
			if (bIsCurClassMethod) {
				ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues), nLineNumber, nBelongingFileIndex);
			}
			// 子类调用父类方法
			else {
				// 禁止调用
				if (pMethod->GetAuthority() == IrisMethod::MethodAuthority::Personal) {
					// **Error**
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, nLineNumber, nBelongingFileIndex, "Method of " + strFunctionName + " is PERSONAL and cannot be called from derrived class " + m_pClass->GetInternClass()->GetClassName() + ".");
					ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
				}
				else {
					ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues), nLineNumber, nBelongingFileIndex);
				}
			}
		}
		// 外部调用
		else {
			// 只能调用EveryOne的方法
			// 禁止调用
			if (pMethod->GetAuthority() != IrisMethod::MethodAuthority::Everyone) {
				// **Error**
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, nLineNumber, nBelongingFileIndex,  "Method of " + strFunctionName + " is not EVERYONE and cannot be called outside the class " + m_pClass->GetInternClass()->GetClassName() + " .");
				ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
			}
			else {
				ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues), nLineNumber, nBelongingFileIndex);
			}
		}
	}
	else {
		// **Error**
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, nLineNumber, nBelongingFileIndex, "Method of " + strFunctionName + " not found in class " + m_pClass->GetInternClass()->GetClassName() + ".");
		ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
	}

	return ivResult;
}

const IrisValue & IrisObject::GetInstanceValue(const string & strInstanceValueName, bool & bResult) {
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

void IrisObject::AddInstanceValue(const string & strInstanceValueName, const IrisValue & ivValue) {
	m_iwlInstanceValueWLLock.WriteLock();
	m_mpInstanceVariables.insert(_VariablePair(strInstanceValueName, ivValue));
	m_iwlInstanceValueWLLock.WriteUnlock();
}

IrisObject::~IrisObject() {
	this->m_pClass->NativeFree(m_pNativeObject);

	for (auto iter : m_mpInstanceMethods) {
		delete iter.second;
	}
}