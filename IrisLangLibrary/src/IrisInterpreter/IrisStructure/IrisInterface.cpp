#include "IrisInterpreter/IrisStructure/IrisInterface.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"

IrisInterface::IrisInterface(const string& strInterfaceName, IrisModule* pUpperModule) : m_strInterfaceName(strInterfaceName), m_pUpperModule(pUpperModule) {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Interface")->CreateInstance(nullptr, nullptr);
	((IrisInterfaceBaseTag*)ivValue.GetInstanceNativePointer())->SetInterface(this);
	m_pInterfaceObject = ivValue.GetIrisObject();
}

void IrisInterface::AddInterface(IrisInterface* pInterface) {

	string strFullPath = "";
	IrisModule* pTmpModule = pInterface->m_pUpperModule;
	while (pTmpModule) {
		strFullPath = pTmpModule->GetModuleName() + "::" + strFullPath;
		pTmpModule = pTmpModule->GetUpperModule();
	}
	strFullPath += pInterface->GetInterfaceName();

	m_iwlInfAddingLock.WriteLock();

	if (m_mpInterfaces.find(strFullPath) != m_mpInterfaces.end()) {
		m_mpInterfaces[strFullPath] = pInterface;
	}
	else {
		m_mpInterfaces.insert(_InterfacePair(strFullPath, pInterface));
	}

	m_iwlInfAddingLock.WriteUnlock();
}

void IrisInterface::AddInterfaceFunctionDeclare(const string & strFunctionName, int m_nParameterAmount, bool bHaveHaveVariableParameter) {
	InterfaceFunctionDeclare ifdDeclare{ strFunctionName, m_nParameterAmount, bHaveHaveVariableParameter };
	m_iwlDecAddingLock.WriteLock();
	decltype(m_mpFunctionDeclareMap)::iterator iFunc;
	if ((iFunc = m_mpFunctionDeclareMap.find(strFunctionName)) != m_mpFunctionDeclareMap.end()) {
		m_mpFunctionDeclareMap[strFunctionName] = ifdDeclare;
	}
	else {
		m_mpFunctionDeclareMap.insert(_InterfaceFunctionDeclarePair(strFunctionName, ifdDeclare));
	}
	m_iwlDecAddingLock.WriteUnlock();
}

IrisInterface::~IrisInterface() {
	delete m_pExternInterface;
}

void IrisInterface::ClearJointingInterfaces() {
	m_iwlInfAddingLock.WriteLock();
	m_mpInterfaces.clear();
	m_iwlInfAddingLock.WriteUnlock();
}
