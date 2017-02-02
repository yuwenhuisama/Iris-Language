#include "IrisInterpreter/IrisStructure/IrisObject.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisUnil/IrisValues.h"
#include "IrisFatalErrorHandler.h"

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
IrisValue IrisObject::CallInstanceFunction(const string& strFunctionName, IIrisContextEnvironment* pContextEnvironment, IIrisValues* ivsValues, CallerSide eSide) {
#else
IrisValue IrisObject::CallInstanceFunction(const IrisInternString& strFunctionName, IIrisContextEnvironment* pContextEnvironment, IIrisValues* ivsValues, CallerSide eSide) {
#endif // IR_USE_STL_STRING
	// 先在自己的Instance Functions中寻找对应方法
	IrisMethod* pMethod = nullptr;
	bool bIsCurClassMethod = false;

	// 特殊处理：如果是类对象或者模块对象，则沿着继承链往上查找一直到Class类为止
	if (m_pClass->GetInternClass()->IsClassClass()) {
		bIsCurClassMethod = false;
		IrisObject* pClassObject = this;
		IrisClass* pClass = static_cast<IrisClassBaseTag*>(pClassObject->m_pNativeObject)->GetClass();

		do {
			pClassObject = static_cast<IrisObject*>(pClass->GetClassObject());

			pMethod = pClassObject->GetInstanceMethod(strFunctionName);

			// 根据“类的单例方法可继承”的原则，所有的类都能够调用Class类的单件方法；而又由于所有的类都是Class类的实例，因此所有的类都能调用Class类的实例方法。
			// Class类
			if (!pMethod) {
				pMethod = static_cast<IrisObject*>(m_pClass->GetInternClass()->GetClassObject())->GetInstanceMethod(strFunctionName);
			}

			if (!pMethod) {
				bool bDummy = false;
				pMethod = m_pClass->GetInternClass()->GetMethod(strFunctionName, bDummy);
			}

			// 检查父类包含的模块
			if (!pMethod) {
				auto& hsModules = pClass->GetModules();
				for (auto& module : hsModules) {
					auto pModule = module;
					if (pMethod = pModule->GetModuleInstanceMethod(strFunctionName)) {
						break;
					}
				}
			}

			pClass = static_cast<IrisClassBaseTag*>(pClassObject->m_pNativeObject)->GetClass()->GetSuperClass();

		} while (pClass && !pClass->IsClassClass() && !pMethod);
	}
	else if (m_pClass->GetInternClass()->IsModuleClass()) {
		bIsCurClassMethod = false;

		IrisModule* pModule = static_cast<IrisModuleBaseTag*>(m_pNativeObject)->GetModule();

		pMethod = static_cast<IrisObject*>(pModule->GetModuleObject())->GetInstanceMethod(strFunctionName);

		if(!pMethod) {
			auto& hsModules = pModule->GetModules();
			for (auto& module : hsModules) {
				auto pTmpModule = module;
				if (pMethod = pTmpModule->GetModuleInstanceMethod(strFunctionName)) {
					break;
				}
			}
		}
	}
	else {
		decltype(m_mpInstanceMethods)::iterator iMethod = m_mpInstanceMethods.find(strFunctionName);
		if (!(iMethod == m_mpInstanceMethods.end())) {
			pMethod = iMethod->second;
			bIsCurClassMethod = true;
		}
	}

	if (!pMethod) {
		pMethod = m_pClass->GetInternClass()->GetMethod(strFunctionName, bIsCurClassMethod);
	}

	auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
	IrisValue ivResult;
	if (pMethod) {
		IrisValue ivValue;
		ivValue.SetIrisObject(this);

		// 内部调用
		if (eSide == CallerSide::Inside) {
			// 本类内部调用无限制
			if (bIsCurClassMethod) {
				ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues));
			}
			// 子类调用父类方法
			else {
				// 禁止调用
				if (pMethod->GetAuthority() == IrisMethod::MethodAuthority::Personal) {
					// **Error**
#if IR_USE_STL_STRING
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " is PERSONAL and cannot be called from derrived class " + m_pClass->GetInternClass()->GetClassName() + ".");
#else
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " is PERSONAL and cannot be called from derrived class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + ".");
#endif // IR_USE_STL_STRING
					ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
				}
				else {
					ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues));
				}
			}
		}
		// 外部调用
		else {
			// 只能调用EveryOne的方法
			// 禁止调用
			if (pMethod->GetAuthority() != IrisMethod::MethodAuthority::Everyone) {
				// **Error**
#if IR_USE_STL_STRING
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " is not EVERYONE and cannot be called outside the class " + m_pClass->GetInternClass()->GetClassName() + " .");
#else
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::MethodAuthorityIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " is not EVERYONE and cannot be called outside the class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + " .");
#endif // IR_USE_STL_STRING
				ivResult = IrisInterpreter::CurrentInterpreter()->Nil();
			}
			else {
				ivResult = pMethod->Call(ivValue, static_cast<IrisContextEnvironment*>(pContextEnvironment), static_cast<IrisValues*>(ivsValues));
			}
		}
	}
	else {
		// **Error**
#if IR_USE_STL_STRING
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName + " not found in class " + m_pClass->GetInternClass()->GetClassName() + ".");
#else
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Method of " + strFunctionName.GetSTLString() + " not found in class " + m_pClass->GetInternClass()->GetClassName().GetSTLString() + ".");
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