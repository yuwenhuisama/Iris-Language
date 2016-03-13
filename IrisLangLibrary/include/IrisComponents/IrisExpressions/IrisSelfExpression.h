#ifndef _H_IRISELFEXPRESSION_
#define _H_IRISELFEXPRESSION_
#include "IrisExpression.h"

class IrisSelfExpression :
	public IrisExpression
{
public:

	virtual bool Generate();

	IrisSelfExpression();
	~IrisSelfExpression();
};

#endif