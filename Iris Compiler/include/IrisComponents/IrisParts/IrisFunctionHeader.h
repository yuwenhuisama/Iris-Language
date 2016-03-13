#ifndef _H_IRISFUNCTIONHEADER_
#define _H_IRISFUNCTIONHEADER_
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisFunctionHeader
{
public:
	IrisIdentifier* m_pFunctionName = nullptr;
	IrisList<IrisIdentifier*>* m_pParameters = nullptr;
	IrisIdentifier* m_pVariableParameter = nullptr;
	bool m_bIsClassMethod = false;

public:
	IrisFunctionHeader(IrisIdentifier* pFunctionName, IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameter, bool bIsClassMethod);
	~IrisFunctionHeader();
};

#endif