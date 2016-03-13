#ifndef _H_IRISARRAYEXPRESSION_
#define _H_IRISARRAYEXPRESSION_
//#include "IrisList.h"
#include "IrisExpression.h"
#include "IrisUnil/IrisList.h"

class IrisArrayExpression :
	public IrisExpression
{
protected:
	IrisList<IrisExpression*>* m_pElementList;

public:

	virtual bool Generate();

	IrisArrayExpression(IrisList<IrisExpression*>* pElementList);
	virtual ~IrisArrayExpression();
};

#endif