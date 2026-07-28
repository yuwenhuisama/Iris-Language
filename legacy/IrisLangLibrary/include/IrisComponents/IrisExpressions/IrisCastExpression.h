#ifndef _H_IRISCASTEXPRESSION_
#define _H_IRISCASTEXPRESSION_
#include "IrisExpression.h"

class IrisCastExpression :
	public IrisExpression
{
public:

	virtual bool Generate();

	IrisCastExpression();
	~IrisCastExpression();

	bool Validate() override;
};

#endif // !_H_IRISCASTEXPRESSION_