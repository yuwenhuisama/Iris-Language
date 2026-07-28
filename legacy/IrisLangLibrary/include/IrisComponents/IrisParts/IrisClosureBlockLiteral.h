#ifndef _H_IRISCLOSUREBLOCKLITERAL_
#define _H_IRISCLOSUREBLOCKLITERAL_
#include "IrisUnil/IrisList.h"

#include "IrisComponents/IrisStatements/IrisStatement.h"

class IrisIdentifier;
class IrisClosureBlockLiteral :
	public IrisStatement
{
public:
	IrisList<IrisIdentifier*>* m_pParameters = nullptr;
	IrisList<IrisStatement*>* m_pStatements = nullptr;
	IrisIdentifier* m_pVariableParameter = nullptr;

public:

	virtual bool Generate();

	IrisClosureBlockLiteral(IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameter, IrisList<IrisStatement*>* pStatements);
	virtual ~IrisClosureBlockLiteral();

	virtual bool Validate() override;
};

#endif