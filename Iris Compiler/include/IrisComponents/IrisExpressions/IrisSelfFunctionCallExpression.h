#ifndef _H_IRISSELFFUNCTIONCALLEXPRESSION_
#define _H_IRISSELFFUNCTIONCALLEXPRESSION_
#include "IrisExpression.h"
#include "IrisList.h"

class IrisIdentifier;
class IrisClosureBlockLiteral;

class IrisSelfFunctionCallExpression :
	public IrisExpression
{
protected:
	IrisIdentifier* m_pFunctionName = nullptr;
	IrisList<IrisExpression*>* m_pParameters = nullptr;
	IrisClosureBlockLiteral* m_pClosureBlock = nullptr;

public:

	virtual bool Generate();

	IrisSelfFunctionCallExpression(IrisIdentifier* pFunctionName, IrisList<IrisExpression*>* pParameters, IrisClosureBlockLiteral* pClosureBlock);
	virtual ~IrisSelfFunctionCallExpression();
};

#endif