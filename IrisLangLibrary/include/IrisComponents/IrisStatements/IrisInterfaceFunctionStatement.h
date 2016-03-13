#ifndef _H_IRISINTERFACEFUNCTIONSTATEMENT_
#define _H_IRISINTERFACEFUNCTIONSTATEMENT_
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisInterfaceFunctionStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pFunctionName = nullptr;
	IrisList<IrisIdentifier*>* m_pParameters = nullptr;
	IrisIdentifier* m_pVariableParameter = nullptr;

public:

	virtual bool Generate();

	IrisInterfaceFunctionStatement(IrisIdentifier* pFunctionName, IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameters);
	virtual ~IrisInterfaceFunctionStatement();
};

#endif