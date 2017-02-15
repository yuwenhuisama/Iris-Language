#ifndef _H_IRISGROANSTATEMENT_
#define _H_IRISGROANSTATEMENT_

#include "IrisStatement.h"

class IrisExpression;
class IrisGroanStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pGroanExpression = nullptr;

public:

	virtual bool Generate();

	IrisGroanStatement(IrisExpression* pGroanExpression);
	virtual ~IrisGroanStatement();

	virtual bool Validate() override;
};

#endif