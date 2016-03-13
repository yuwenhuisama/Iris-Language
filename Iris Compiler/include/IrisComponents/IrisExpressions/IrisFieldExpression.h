#ifndef _H_IRISFILDEXPRESSION_
#define _H_IRISFILDEXPRESSION_

#include "IrisExpression.h"

class IrisFieldIdentifier;
class IrisFieldExpression :
	public IrisExpression
{
protected:
	IrisFieldIdentifier* m_pFieldIdentifier = nullptr;

public:

	virtual bool Generate();

	IrisFieldExpression(IrisFieldIdentifier* pFieldIdentifier);
	virtual ~IrisFieldExpression();
};

#endif