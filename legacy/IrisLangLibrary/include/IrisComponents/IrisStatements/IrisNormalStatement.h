#ifndef _H_IRISNORMALSTATEMENT_
#define _H_IRISNORMALSTATEMENT_
#include "IrisStatement.h"

class IrisExpression;
class IrisNormalStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pExpression = nullptr;

public:

	virtual bool Generate();

	IrisNormalStatement(IrisExpression* pExpression);
	virtual ~IrisNormalStatement();

	virtual bool Validate() override;
};

#endif