#include "IrisComponents/IrisParts/IrisFunctionHeader.h"
#include "IrisUnil/IrisIdentifier.h"


IrisFunctionHeader::IrisFunctionHeader(IrisIdentifier* pFunctionName, IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameter, bool bIsClassMethod) : m_pFunctionName(pFunctionName), m_pParameters(pParameters), m_pVariableParameter(pVariableParameter), m_bIsClassMethod(bIsClassMethod)
{
}


IrisFunctionHeader::~IrisFunctionHeader()
{
	if (m_pFunctionName)
		delete m_pFunctionName;
	if (m_pParameters) {
		m_pParameters->Ergodic([](IrisIdentifier*& x) -> bool { delete x; x = nullptr; return true; });
		m_pParameters->Clear();
		delete m_pParameters;
	}
	if (m_pVariableParameter)
		delete m_pVariableParameter;
}
