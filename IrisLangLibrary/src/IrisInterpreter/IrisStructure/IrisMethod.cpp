#include "IrisInterpreter/IrisStructure/IrisMethod.h"
#include "IrisInterpreter/IrisNativeClasses/IrisMethodBase.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisParts/IrisFunctionHeader.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisValues.h"
#include "IrisFatalErrorHandler.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"

//#include "IrisInterpreter/IrisNativeClasses/IrisIrregularTag.h"

#if IR_USE_STL_STRING
IrisMethod::IrisMethod(const string& strMethodName, IrisNativeFunction pfNativeFunction, int nParameterAmount, bool bIsWithVariableParameter, MethodAuthority eAuthority) {
#else
IrisMethod::IrisMethod(const IrisInternString& strMethodName, IrisNativeFunction pfNativeFunction, int nParameterAmount, bool bIsWithVariableParameter, MethodAuthority eAuthority) {
#endif // IR_USE_STL_STRING
	m_strMethodName = strMethodName;
	m_eMethodType = MethodType::NativeMethod;
	m_uFunction.m_pfNativeFunction = pfNativeFunction;
	m_bIsWithVariableParameter = bIsWithVariableParameter;
	m_nParameterAmount = nParameterAmount;
	m_eAuthority = eAuthority;

	IrisClass* pMethodClass = nullptr;
	if (pMethodClass = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Method")) {
		IrisValue ivValue = pMethodClass->CreateInstance(nullptr, nullptr);
		IrisDevUtil::GetNativePointer<IrisMethodBaseTag*>(ivValue)->SetMethod(this);
		m_pMethodObject = ivValue.GetIrisObject();
	}
}

#if IR_USE_STL_STRING
IrisMethod::IrisMethod(const string& strMethodName, UserFunction* pUserFunction, MethodAuthority eAuthority) {
#else
IrisMethod::IrisMethod(const IrisInternString& strMethodName, UserFunction* pUserFunction, MethodAuthority eAuthority) {
#endif // IR_USE_STL_STRING
	m_strMethodName = strMethodName;
	m_eMethodType = MethodType::UserMethod;
	m_uFunction.m_pUserFunction = pUserFunction;
	m_bIsWithVariableParameter = (pUserFunction->m_strVariableParameter != "");
	m_eAuthority = eAuthority;

	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Method")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisMethodBaseTag*>(ivValue)->SetMethod(this);
	m_pMethodObject = ivValue.GetIrisObject();
	m_nParameterAmount = pUserFunction->m_lsParameters.size();
}

#if IR_USE_STL_STRING
IrisMethod::IrisMethod(const string & strMethodName, UserFunction* pUserFunction, MethodType eType, MethodAuthority eAuthority)
#else
IrisMethod::IrisMethod(const IrisInternString & strMethodName, UserFunction* pUserFunction, MethodType eType, MethodAuthority eAuthority)
#endif // IR_USE_STL_STRING
{
	m_strMethodName = strMethodName;
	m_eMethodType = eType;
	m_uFunction.m_pUserFunction = pUserFunction;
	m_bIsWithVariableParameter = false;
	m_eAuthority = eAuthority;

	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Method")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisMethodBaseTag*>(ivValue.GetIrisObject())->SetMethod(this);
	m_pMethodObject = ivValue.GetIrisObject();
	m_nParameterAmount = pUserFunction->m_lsParameters.size();
}

bool IrisMethod::_ParameterCheck(IrisValues* pParameters) {
	// �������
	// ����пɱ��������ôʵ�ʲ����ĸ�������Ҫ���ڵ�����ʽ����
	if (pParameters) {
		if (m_bIsWithVariableParameter) {
			return pParameters->GetSize() >= (unsigned int) m_nParameterAmount;
		}
		// ���û�пɱ��������ô��ʵ�ʲ�������Ҫ������ʽ����
		else {
			return pParameters->GetSize() == m_nParameterAmount;
		}
	}
	else {
		return m_nParameterAmount == 0;
	}
}

IrisContextEnvironment* IrisMethod::_CreateContextEnvironment(IrisObject* pCaller, IrisValues* pParameters, IrisContextEnvironment* pContextEnvrioment, bool& bIsGetNew) {

	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	//IrisContextEnvironment* pNewEnvironment = new IrisContextEnvironment();
	IrisContextEnvironment* pNewEnvironment = pContextEnvrioment;

	bool bInitialize = false;
	bIsGetNew = false;
	if (!pNewEnvironment) {
		pNewEnvironment = new IrisContextEnvironment();
		pNewEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::Runtime;
		pNewEnvironment->m_uType.m_pCurObject = pCaller;

		pNewEnvironment->m_pUpperContextEnvironment = IrisInterpreter::CurrentInterpreter()->GetCurrentContextEnvrionment();

		bInitialize = true;
		bIsGetNew = true;
	}

	//pNewEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::Runtime;
	pNewEnvironment->m_uType.m_pCurObject = pCaller;
	pNewEnvironment->m_pCurMethod = this;

	// With Block and Without Block
	if (m_eMethodType == MethodType::UserMethod) {
		pNewEnvironment->m_pWithBlock = !m_uFunction.m_pUserFunction->m_icsWithBlockCodes.m_pWholeCodes ? nullptr : &m_uFunction.m_pUserFunction->m_icsWithBlockCodes;
		pNewEnvironment->m_pWithoutBlock = !m_uFunction.m_pUserFunction->m_icsWithoutBlockCodes.m_pWholeCodes ? nullptr : &m_uFunction.m_pUserFunction->m_icsWithoutBlockCodes;
	}

	// ����÷�����UserFunction�򽫲�����Ϊ�ֲ�������ӽ���
	if (pParameters && m_eMethodType == MethodType::UserMethod) {
		auto it = pParameters->GetVector().begin();
		for (auto param : m_uFunction.m_pUserFunction->m_lsParameters) {
			pNewEnvironment->AddLocalVariable(param, *(it++));
		}
		// ����пɱ����
		if (m_bIsWithVariableParameter) {
			// �½�һ������
			IrisValues ivsValues;
			ivsValues.GetVector().assign(it, pParameters->GetVector().end());
			auto pClass = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Array");
			IrisValue ivArray = pClass->CreateInstance(&ivsValues, nullptr);
			pNewEnvironment->AddLocalVariable(m_uFunction.m_pUserFunction->m_strVariableParameter, ivArray);
		}
	}
	// ContextEnvironment���

	if (bInitialize) {
		IrisGC::CurrentGC()->AddContextEnvironmentSize();
		auto pEnvRegister = IrisDevUtil::GetCurrentThreadInfo()->m_pEnvrionmentRegister;
		if(pEnvRegister) {
			++pEnvRegister->m_nReferenced;
			IrisGC::CurrentGC()->ContextEnvironmentGC();
			--pEnvRegister->m_nReferenced;
		}
		else {
			IrisGC::CurrentGC()->ContextEnvironmentGC();
		}
		IrisInterpreter::CurrentInterpreter()->PushEnvironment();
		IrisInterpreter::CurrentInterpreter()->SetEnvironment(pNewEnvironment);
		IrisInterpreter::CurrentInterpreter()->AddNewEnvironmentToHeap(pNewEnvironment);
	}
	return pNewEnvironment;
}

// �������͵Ĳ�ͬ�ֱ���ò�ͬ�ĺ�����ֱ�ӵ��� or �������У�
IrisValue IrisMethod::Call(IrisValue& ivObject, IrisContextEnvironment* pContextEnvironment ,IrisValues* pParameters) {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->Nil();
	IrisValues ivsNormalPrameters;
	IrisValues ivsVariableValues;
	bool bHaveVariableParameters = false;
	auto pInfo = IrisDevUtil::GetCurrentThreadInfo();

	// ����������
	// ������� new ����飬����__format���
	if (!_ParameterCheck(pParameters)) {
		// **Error**
#if IR_USE_STL_STRING
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ParameterNotFitIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Parameters of method " + m_strMethodName + " assigned is not fit.");
#else
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ParameterNotFitIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Parameters of method " + m_strMethodName.GetSTLString() + " assigned is not fit.");
#endif // IR_USE_STL_STRING
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}

	// Getter Setter
	if (m_eMethodType == MethodType::GetterMethod) {
		string strValueName;
#if IR_USE_STL_STRING
		strValueName.assign(m_strMethodName.begin() + 6, m_strMethodName.end());
#else
		strValueName.assign(m_strMethodName.GetSTLString().begin() + 6, m_strMethodName.GetSTLString().end());
#endif // IR_USE_STL_STRING
		strValueName = "@" + strValueName;
		bool bResult = false;
		auto* pObject = static_cast<IrisObject*>(ivObject.GetIrisObject());
		const IrisValue& ivResult = pObject->GetInstanceValue(strValueName, bResult);
		// ����������
		if (!bResult) {
			pObject->AddInstanceValue(strValueName, IrisInterpreter::CurrentInterpreter()->Nil());
			return IrisInterpreter::CurrentInterpreter()->Nil();
		}
		return ivResult;
	}
	else if (m_eMethodType == MethodType::SetterMethod) {
		string strValueName;
#if IR_USE_STL_STRING
		strValueName.assign(m_strMethodName.begin() + 6, m_strMethodName.end());
#else
		strValueName.assign(m_strMethodName.GetSTLString().begin() + 6, m_strMethodName.GetSTLString().end());
#endif // IR_USE_STL_STRING
		strValueName = "@" + strValueName;
		bool bResult = false;
		auto pObject = static_cast<IrisObject*>(ivObject.GetIrisObject());
		IrisValue& ivResult = (IrisValue&)pObject->GetInstanceValue(strValueName, bResult);
		IrisValue& ivParam = (*pParameters)[0];
		// ����������
		if (!bResult) {
			pObject->AddInstanceValue(strValueName, ivParam);
			return ivParam;
		}
		ivResult.SetIrisObject(ivParam.GetIrisObject());
		return ivParam;
	}

	bool bIsGetNew = false;
	IrisContextEnvironment* pNewEnvironment = _CreateContextEnvironment(static_cast<IrisObject*>(ivObject.GetIrisObject()), pParameters, pContextEnvironment, bIsGetNew);
	++pNewEnvironment->m_nReferenced;
	//pNewEnvironment->m_pUpperContextEnvironment = pContextEnvironment;
	
	//�������Ϊ�գ�ֱ�ӵ���
	if (!pParameters){
		if (m_eMethodType == MethodType::NativeMethod) {
			ivValue = m_uFunction.m_pfNativeFunction(ivObject, nullptr, nullptr, pNewEnvironment);
		}
		else {
			// ���������뻷����
			//ivValue = m_uFunction.m_pUserFunction->m_pBlock->Execute(pNewEnvironment).m_ivValue;
			IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
			//IrisAM iaAM = pInterpreter->GetOneAM(iCoderPointer);
			pInterpreter->PushMethodDeepIndex(m_uFunction.m_pUserFunction->m_dwIndex);

			auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
			auto nOldFileIndex = pInfo->m_nCurrentFileIndex;
			pInfo->m_nCurrentFileIndex = m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex;
			pInterpreter->RunCode(*m_uFunction.m_pUserFunction->m_icsBlockCodes.m_pWholeCodes, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nStartPointer, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nEndPointer);
			pInfo->m_nCurrentFileIndex = nOldFileIndex;

			ivValue = IrisInterpreter::CurrentInterpreter()->GetCurrentResultRegister();
			pInterpreter->PopMethodTopDeepIndex();
		}
	}
	else {
		// ���ɱ����
		if (pParameters->GetSize() > m_nParameterAmount) {
			bHaveVariableParameters = true;
			ivsVariableValues.GetVector().assign(pParameters->GetVector().begin() + m_nParameterAmount, pParameters->GetVector().end());
		}
		if (m_nParameterAmount > 0) {
			ivsNormalPrameters.GetVector().assign(pParameters->GetVector().begin(), pParameters->GetVector().begin() + m_nParameterAmount);
		}

		if (m_eMethodType == MethodType::NativeMethod) {
			++IrisDevUtil::GetCurrentThreadInfo()->m_nNativeReference;
			if (bHaveVariableParameters) {
				if (m_nParameterAmount > 0) {
					ivValue = m_uFunction.m_pfNativeFunction(ivObject, &ivsNormalPrameters, &ivsVariableValues, pNewEnvironment);
				}
				else {
					ivValue = m_uFunction.m_pfNativeFunction(ivObject, nullptr, &ivsVariableValues, pNewEnvironment);
				}
			}
			else {
				if (m_nParameterAmount > 0) {
					ivValue = m_uFunction.m_pfNativeFunction(ivObject, &ivsNormalPrameters, nullptr, pNewEnvironment);
				}
				else {
					ivValue = m_uFunction.m_pfNativeFunction(ivObject, nullptr, nullptr, pNewEnvironment);
				}
			}
			--IrisDevUtil::GetCurrentThreadInfo()->m_nNativeReference;
			if (IrisDevUtil::GetCurrentThreadInfo()->m_nNativeReference == 0) {
				//for (auto& obj : IrisDevUtil::GetCurrentThreadInfo()->m_hpObjectInNativeFunctionHeap) {
				//	obj->SetIsCreateInNativeFunction(false);
				//}
				IrisDevUtil::GetCurrentThreadInfo()->m_hpObjectInNativeFunctionHeap.clear();
			}
		}
		else {
			// ���������뻷����
			IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
			
			pInterpreter->PushMethodDeepIndex(m_uFunction.m_pUserFunction->m_dwIndex);

			auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
			auto nOldFileIndex = pInfo->m_nCurrentFileIndex;
			pInfo->m_nCurrentFileIndex = m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex;
			pInterpreter->RunCode(*m_uFunction.m_pUserFunction->m_icsBlockCodes.m_pWholeCodes, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nStartPointer, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nEndPointer);
			pInfo->m_nCurrentFileIndex = nOldFileIndex;

			ivValue = IrisInterpreter::CurrentInterpreter()->GetCurrentResultRegister();
			pInterpreter->PopMethodTopDeepIndex();
		}
	}
	if (bIsGetNew) {
		IrisInterpreter::CurrentInterpreter()->PopEnvironment();
	}

	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	if (pInterpreter->IrregularHappened()) {
		auto p = IrisThreadManager::CurrentThreadManager();
		if(pParameters) {
			//auto pInfo = IrisDevUtil::GetCurrentThreadInfo();
			//auto strString = IrisDevUtil::GetNativePointer<IrisIrregularTag*>(pInfo->m_ivIrregularObjectRegister);

			pInterpreter->PopStack(pParameters->GetVector().size());
		}
	}

	--pNewEnvironment->m_nReferenced;

	//if (!pNewEnvironment->m_pClosureBlock) {
	//	delete pNewEnvironment->m_pClosureBlock;
	//	pNewEnvironment->m_pClosureBlock = nullptr;
	//}

	return ivValue;
}

IrisValue IrisMethod::CallMainMethod(IrisValues* pParameters) {
	IrisValue ivValue;

	IrisValues ivsNormalPrameters;
	IrisValues ivsVariableValues;
	bool bHaveVariableParameters = false;

	//if (!strBelongingFileName) {
	//	strBelongingFileName = &IrisDevUtil::GetCurrentThreadInfo()->m_strCurrentFileName;
	//}

	auto pInfo = IrisDevUtil::GetCurrentThreadInfo();

	// ����������
	if (!_ParameterCheck(pParameters)) {
		// **Error**
#if IR_USE_STL_STRING
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ParameterNotFitIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Parameters of method " + m_strMethodName + " assigned is not fit.");
#else
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ParameterNotFitIrregular, pInfo->m_nCurrentLineNumber, pInfo->m_nCurrentFileIndex, "Parameters of method " + m_strMethodName.GetSTLString() + " assigned is not fit.");
#endif // IR_USE_STL_STRING
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
	bool bIsGetNew = false;
	IrisContextEnvironment* pNewEnvironment = _CreateContextEnvironment(nullptr, pParameters, IrisInterpreter::CurrentInterpreter()->GetCurrentContextEnvrionment(), bIsGetNew);	
	++pNewEnvironment->m_nReferenced;
	//pNewEnvironment->m_pUpperContextEnvironment = nullptr;

	// ���������뻷����
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	
	pInterpreter->PushMethodDeepIndex(m_uFunction.m_pUserFunction->m_dwIndex);

	auto nOldFileIndex = pInfo->m_nCurrentFileIndex;
	pInfo->m_nCurrentFileIndex = m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex;
	pInterpreter->RunCode(*m_uFunction.m_pUserFunction->m_icsBlockCodes.m_pWholeCodes, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nStartPointer, m_uFunction.m_pUserFunction->m_icsBlockCodes.m_nEndPointer);
	pInfo->m_nCurrentFileIndex = nOldFileIndex;

	if (bIsGetNew) {
		IrisInterpreter::CurrentInterpreter()->PopEnvironment();
	}
	pInterpreter->PopMethodTopDeepIndex();

	if (pInterpreter->IrregularHappened()) {
		if (pParameters) {
			pInterpreter->PopStack(pParameters->GetVector().size());
		}
	}

	--pNewEnvironment->m_nReferenced;

	//if (!pNewEnvironment->m_nReferenced && pNewEnvironment->m_pClosureBlock) {
	//	delete pNewEnvironment->m_pClosureBlock;
	//	pNewEnvironment->m_pClosureBlock = nullptr;
	//}

	return IrisInterpreter::CurrentInterpreter()->GetCurrentResultRegister();
}

void IrisMethod::ResetObject() {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Method")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisMethodBaseTag*>(ivValue)->SetMethod(this);
	m_pMethodObject = ivValue.GetIrisObject();
}

void IrisMethod::SetMethodAuthority(MethodAuthority eAuthority) {
	m_eAuthority = eAuthority;
}

IrisMethod::~IrisMethod() {
	if (m_eMethodType == MethodType::UserMethod){
		delete m_uFunction.m_pUserFunction;
	}
	//delete m_pMethodObject;
}

#if IR_USE_STL_STRING
const string & IrisMethod::GetMethodName() {
#else
const IrisInternString & IrisMethod::GetMethodName() {
#endif // IR_USE_STL_STRING
	return m_strMethodName;
}

#if IR_USE_STL_STRING
void IrisMethod::SetMethodName(const string & strMethodName) {
#else
void IrisMethod::SetMethodName(const IrisInternString & strMethodName) {
#endif // IR_USE_STL_STRING
	m_strMethodName = strMethodName;
}

IrisMethod::MethodAuthority IrisMethod::GetAuthority() {
	return m_eAuthority;
}
