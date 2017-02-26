#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include "IrisFatalErrorHandler.h"

IrisContextEnvironment* IrisClosureBlock::_CreateNewContextEnvironment(IIrisThreadInfo* pThreadInfo) {
	IrisContextEnvironment* pNewEnvironment = new IrisContextEnvironment();
	pNewEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::Runtime;
	pNewEnvironment->m_uType.m_pCurObject = m_pNativeObject;

	IrisGC::CurrentGC()->AddContextEnvironmentSize();
	IrisGC::CurrentGC()->ContextEnvironmentGC();

	pNewEnvironment->m_bIsClosure = true;
	pNewEnvironment->m_pClosureBlock = this;
	//pNewEnvironment->m_bIsReferenced = true;
	pNewEnvironment->m_pUpperContextEnvironment = m_pUpperContextEnvironment;
	auto pTemp = pNewEnvironment;
	while (pTemp) {
		++pTemp->m_nReferenced;
		pTemp = pTemp->m_pUpperContextEnvironment;
	}

	IrisInterpreter::CurrentInterpreter()->AddNewEnvironmentToHeap(pNewEnvironment, static_cast<IrisThreadInfo*>(pThreadInfo));
	return pNewEnvironment;
}

#if IR_USE_STL_STRING
const IrisValue& IrisClosureBlock::GetLocalVariable(const string& strVariableName, bool& bResult) {
#else
const IrisValue& IrisClosureBlock::GetLocalVariable(const IrisInternString& strVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
	//IrisValue ivResult{ IrisInterpreter::CurrentInterpreter()->Nil() };
	IrisValue* pValue = nullptr;

	bResult = false;
	IrisContextEnvironment* pTmpContextEnvrionment = m_pCurContextEnvironment;
	while (pTmpContextEnvrionment && !bResult) {
		pValue = (IrisValue*)&(pTmpContextEnvrionment->GetVariableValue(strVariableName, bResult));
		pTmpContextEnvrionment = pTmpContextEnvrionment->m_pUpperContextEnvironment;
	}

	if (bResult) {
		return *pValue;
	}
	else {
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
}
#if IR_USE_STL_STRING
const IrisValue& IrisClosureBlock::GetInstanceVariable(const string& strVariableName, bool& bResult) {
#else
const IrisValue& IrisClosureBlock::GetInstanceVariable(const IrisInternString& strVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
	//IrisValue ivResult{ IrisInterpreter::CurrentInterpreter()->Nil() };
	bResult = false;
	IrisValue* pValue = nullptr;

	decltype(m_mpOtherVariables)::iterator iValue;
	m_iwlVariableLock.ReadLock();
	if ((iValue = m_mpOtherVariables.find(strVariableName)) != m_mpOtherVariables.end()) {
		bResult = true;
		auto& ivResult = iValue->second;
		m_iwlVariableLock.ReadUnlock();
		return ivResult;
	}
	else {
		m_iwlVariableLock.ReadUnlock();
		IrisContextEnvironment* pContextEnvrionment = m_pUpperContextEnvironment;
		while (pContextEnvrionment) {
			IrisObject* pObject = pContextEnvrionment->m_uType.m_pCurObject;
			if (pObject) {
				if (pObject->GetClass()->GetInternClass()->IsNormalClass()) {
					pValue = const_cast<IrisValue*>((&pObject->GetInstanceValue(strVariableName, bResult)));
				}
			}
			else {
				pValue = const_cast<IrisValue*>(&pContextEnvrionment->GetVariableValue(strVariableName, bResult));
			}
			if (bResult) {
				return *pValue;
			}
			pContextEnvrionment = pContextEnvrionment->m_pUpperContextEnvironment;
		}

		if (bResult) {
			return *pValue;
		}
		else {
			return IrisInterpreter::CurrentInterpreter()->Nil();
		}
	}
}

#if IR_USE_STL_STRING
const IrisValue& IrisClosureBlock::GetClassVariable(const string& strVariableName, bool& bResult) {
#else
const IrisValue& IrisClosureBlock::GetClassVariable(const IrisInternString& strVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
	//IrisValue ivResult{ IrisInterpreter::CurrentInterpreter()->Nil() };
	bResult = false;
	IrisValue* pValue = nullptr;

	decltype(m_mpOtherVariables)::iterator iValue;
	m_iwlVariableLock.ReadLock();
	if ((iValue = m_mpOtherVariables.find(strVariableName)) != m_mpOtherVariables.end()) {
		bResult = true;
		auto& ivResult = iValue->second;
		m_iwlVariableLock.ReadUnlock();
		return ivResult;
	}
	else {
		m_iwlVariableLock.ReadUnlock();
		IrisContextEnvironment* pContextEnvrionment = m_pUpperContextEnvironment;
		while (pContextEnvrionment) {
			IrisObject* pObject = pContextEnvrionment->m_uType.m_pCurObject;
			if (pObject) {
				if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
					pValue = const_cast<IrisValue*>(&((IrisClassBaseTag*)pObject->GetNativeObject())->GetClass()->SearchClassVariable(strVariableName, bResult));
				}
				else if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
					pValue = const_cast<IrisValue*>(&((IrisModuleBaseTag*)pObject->GetNativeObject())->GetModule()->SearchClassVariable(strVariableName, bResult));
				}
				else {
					pValue = const_cast<IrisValue*>(&pObject->GetClass()->GetInternClass()->SearchClassVariable(strVariableName, bResult));
				}

			}
			else {
				pValue = const_cast<IrisValue*>(&pContextEnvrionment->GetVariableValue(strVariableName, bResult));
			}
			if (bResult) {
				return *pValue;
			}

			pContextEnvrionment = pContextEnvrionment->m_pUpperContextEnvironment;
		}

		if (bResult) {
			return *pValue;
		}
		else {
			return IrisInterpreter::CurrentInterpreter()->Nil();
		}
	}
}
#if IR_USE_STL_STRING
const IrisValue& IrisClosureBlock::GetConstance(const string& strConstanceName, bool& bResult) {
#else
const IrisValue& IrisClosureBlock::GetConstance(const IrisInternString& strConstanceName, bool& bResult) {
#endif // IR_USE_STL_STRING
	//IrisValue ivResult{ IrisInterpreter::CurrentInterpreter()->Nil() };
	IrisValue* pValue = nullptr;
	bResult = false;
	IrisContextEnvironment* pContextEnvrionment = m_pUpperContextEnvironment;
	while (pContextEnvrionment) {
		IrisObject* pObject = pContextEnvrionment->m_uType.m_pCurObject;
		if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
			pValue = (IrisValue*)&((IrisClassBaseTag*)pObject->GetNativeObject())->GetClass()->SearchConstance(strConstanceName, bResult);
		}
		else if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
			pValue = (IrisValue*)&((IrisModuleBaseTag*)pObject->GetNativeObject())->GetModule()->SearchConstance(strConstanceName, bResult);
		}
		else {
			pValue = (IrisValue*)&pObject->GetClass()->GetInternClass()->SearchConstance(strConstanceName, bResult);
		}
		if (bResult) {
			return *pValue;
		}
		pContextEnvrionment = pContextEnvrionment->m_pUpperContextEnvironment;
	}

	if (bResult) {
		return *pValue;
	}
	else {
		pValue = (IrisValue*)&IrisInterpreter::CurrentInterpreter()->GetConstance(strConstanceName, bResult);
		if (bResult) {
			return *pValue;
		}
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
}

void IrisClosureBlock::Mark() {
	for (auto value : m_mpOtherVariables) {
		static_cast<IrisObject*>(value.second.GetIrisObject())->Mark();
	}
}

#if IR_USE_STL_STRING
void IrisClosureBlock::AddLocalVariable(const string& strVariableName, const IrisValue& ivValue) {
#else
void IrisClosureBlock::AddLocalVariable(const IrisInternString& strVariableName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
	m_pCurContextEnvironment->AddLocalVariable(strVariableName, ivValue);
}

#if IR_USE_STL_STRING
void IrisClosureBlock::AddOtherVariable(const string & strVariableName, const IrisValue & ivValue) {
#else
void IrisClosureBlock::AddOtherVariable(const IrisInternString & strVariableName, const IrisValue & ivValue) {
#endif // IR_USE_STL_STRING
	m_iwlVariableLock.WriteLock();
	decltype(m_mpOtherVariables)::iterator iValue;
	if (m_mpOtherVariables.find(strVariableName) != m_mpOtherVariables.end()) {
		m_mpOtherVariables[strVariableName] = ivValue;
	}
	else {
		m_mpOtherVariables.insert(_VariablePair(strVariableName, ivValue));
	}
	m_iwlVariableLock.WriteUnlock();
}

bool IrisClosureBlock::_ParameterCheck(IrisValues* pParameters)
{
	if (pParameters) {
		if (m_strVariableParameter != "") {
			return pParameters->GetSize() >= m_lsParameters.size();
		}
		else {
			return pParameters->GetSize() == m_lsParameters.size();
		}
	}
	else {
		return m_lsParameters.empty();
	}
}

IrisValue IrisClosureBlock::Excute(IIrisValues* pValues, IIrisThreadInfo* pThreadInfo) {

	auto pInfo = static_cast<IrisThreadInfo*>(pThreadInfo);
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	auto& vcCodes = *m_icsCodes.m_pWholeCodes;
	auto& nStartPointer = m_icsCodes.m_nStartPointer;
	auto& nEndPointer = m_icsCodes.m_nEndPointer;
	auto& nBelongingFileIndex = m_icsCodes.m_nBelongingFileIndex;
	
	// Parameter check
	if (!_ParameterCheck(static_cast<IrisValues*>(pValues))) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ParameterNotFitIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Parameters of block assigned is not fit.", pInfo);
		return pInterpreter->Nil();
	}

	if (pValues) {
		if (!m_lsParameters.empty()) {
			vector<IrisValue>::iterator it = static_cast<IrisValues*>(pValues)->GetVector().begin();
			for (auto parm : m_lsParameters) {
				bool bResult = false;
				IrisValue& ivValue = const_cast<IrisValue&>(m_pCurContextEnvironment->GetVariableValue(parm, bResult));
				if (bResult) {
					ivValue.SetIrisObject(((IrisValue&)*(it++)).GetIrisObject());
				}
				else {
					m_pCurContextEnvironment->AddLocalVariable(parm, (const IrisValue)*(it++));
				}
			}
		}
		if (m_strVariableParameter != "") {
			vector<IrisValue>::iterator it = static_cast<IrisValues*>(pValues)->GetVector().begin() + m_lsParameters.size();

			auto pClass = IrisDevUtil::GetClass("Array")->GetInternClass();
			IrisValues ivsValues;
			ivsValues.GetVector().assign(it, static_cast<IrisValues*>(pValues)->GetVector().end());

			auto ivArray = pClass->CreateInstance(&ivsValues, nullptr, pThreadInfo);
			bool bResult = false;
			IrisValue& ivValue = const_cast<IrisValue&>(m_pCurContextEnvironment->GetVariableValue(m_strVariableParameter, bResult));
			if (bResult) {
				ivValue.SetIrisObject(ivArray.GetIrisObject());
			}
			else {
				m_pCurContextEnvironment->AddLocalVariable(m_strVariableParameter, ivArray);
			}

		}
	}

	IrisValue ivValue;
	IrisInterpreter::CurrentInterpreter()->PushEnvironment(pInfo);
	IrisInterpreter::CurrentInterpreter()->SetEnvironment(m_pCurContextEnvironment, pInfo);
	if (!vcCodes.empty()) {
		IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

		//IrisAM iaAM = pInterpreter->GetOneAM(iCoderPointer);
		//pInterpreter->PushMethodDeepIndex(m_nIndex, pInfo);

		auto nOldFileIndex = pInfo->m_nCurrentFileIndex;
		pInfo->m_nCurrentFileIndex = nBelongingFileIndex;
		pInterpreter->RunCode(vcCodes, nStartPointer, nEndPointer, pInfo);
		pInfo->m_nCurrentFileIndex = nOldFileIndex;

		ivValue = IrisInterpreter::CurrentInterpreter()->GetCurrentResultRegister(pInfo);
		//pInterpreter->PopMethodTopDeepIndex(pInfo);
	}
	IrisInterpreter::CurrentInterpreter()->PopEnvironment(pInfo);

	//m_pCurContextEnvironment->m_pExcuteBlock = nullptr;

	return ivValue;
}

void IrisClosureBlock::SetNativeObject(IrisObject * pObject)
{
	m_pNativeObject = pObject;
	m_pCurContextEnvironment->m_uType.m_pCurObject = pObject;
}

IrisObject * IrisClosureBlock::GetNativeObject()
{
	return m_pNativeObject;
}

#if IR_USE_STL_STRING
IrisClosureBlock::IrisClosureBlock(IrisContextEnvironment* pUpperContexEnvironment, list<string>& lsParameters, const string& strVariableParamterName, unsigned int nStartPointer, unsigned int nEndPointer, vector<IR_WORD>& lsCodes, int nBelongingFileIndex, unsigned int nIndex, IrisThreadInfo* pThreadInfo) : m_strVariableParameter(strVariableParamterName), m_pUpperContextEnvironment(pUpperContexEnvironment), m_mpOtherVariables(), m_nIndex(nIndex)
#else
IrisClosureBlock::IrisClosureBlock(IrisContextEnvironment* pUpperContexEnvironment, list<IrisInternString>& lsParameters, const IrisInternString& strVariableParamterName, unsigned int nStartPointer, unsigned int nEndPointer, vector<IR_WORD>& lsCodes, int nBelongingFileIndex, unsigned int nIndex, IrisThreadInfo* pThreadInfo) : m_strVariableParameter(strVariableParamterName), m_pUpperContextEnvironment(pUpperContexEnvironment), m_mpOtherVariables(), m_nIndex(nIndex)
#endif // IR_USE_STL_STRING
{
	m_lsParameters.assign(lsParameters.begin(), lsParameters.end());
	//m_vcCodes.assign(lsCodes.begin(), lsCodes.end());

	m_icsCodes.m_pWholeCodes = &lsCodes;
	m_icsCodes.m_nStartPointer = nStartPointer;
	m_icsCodes.m_nEndPointer = nEndPointer;
	m_icsCodes.m_nBelongingFileIndex = nBelongingFileIndex;

	m_pCurContextEnvironment = _CreateNewContextEnvironment(pThreadInfo);

}

IrisClosureBlock::~IrisClosureBlock()
{
	auto pTemp = m_pCurContextEnvironment;
	while (pTemp) {
		--pTemp->m_nReferenced;
		pTemp = pTemp->m_pUpperContextEnvironment;
	}
}