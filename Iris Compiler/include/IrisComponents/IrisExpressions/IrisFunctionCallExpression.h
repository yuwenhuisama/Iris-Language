#ifndef _H_IRISFUNCTIONCALLEXPRESSION_
#define _H_IRISFUNCTIONCALLEXPRESSION_
#include "IrisExpression.h"
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisClosureBlockLiteral;

class IrisFunctionCallExpression :
	public IrisExpression
{
protected:
	IrisExpression* m_pObject = nullptr;
	IrisIdentifier* m_pFunctionName = nullptr;
	IrisList<IrisExpression*>* m_pParameters = nullptr;
	IrisClosureBlockLiteral* m_pClosureBlock = nullptr;

public:

	virtual bool Generate();

	IrisFunctionCallExpression(IrisExpression* pObject, IrisIdentifier* pFunctionName, IrisList<IrisExpression*>* pParameters, IrisClosureBlockLiteral* pClosureBlock);
	virtual ~IrisFunctionCallExpression();
};

#endif