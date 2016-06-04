#include "IrisInterpreter/IrisStructure/IrisInterface.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisDevelopUtil.h"

IrisInterface::IrisInterface(const IrisInternString& strInterfaceName, IrisModule* pUpperModule) : m_strInterfaceName(strInterfaceName), m_pUpperModule(pUpperModule) {
	IrisValue ivValue = IrisInterpreter::CurrentInterpreter()->GetIrisClass("Interface")->CreateInstance(nullptr, nullptr);
	IrisDevUtil::GetNativePointer<IrisInterfaceBaseTag*>(ivValue)->SetInterface(this);
	m_pInterfaceObject = ivValue.GetIrisObject();
}

void IrisInterface::AddInterface(IrisInterface* pInterface) {
	m_iwlInfAddingLock.WriteLock();
	m_mpInterfaces.insert(pInterface);
	m_iwlInfAddingLock.WriteUnlock();
}

void IrisInterface::AddInterfaceFunctionDeclare(const IrisInternString & strFunctionName, int m_nParameterAmount, bool bHaveHaveVariableParameter) {
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
