#ifndef _H_IRISRETURNSTATEMENT_
#define _H_IRISRETURNSTATEMENT_
#include "IrisStatement.h"

class IrisExpression;
class IrisReturnStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pReturnExpression = nullptr;

public:

	virtual bool Generate();

	IrisReturnStatement(IrisExpression* pReturnExpression);
	virtual ~IrisReturnStatement();

	virtual bool Validate() override;
};

#endif