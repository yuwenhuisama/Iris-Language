#ifndef _H_IRISINSTANTVALUEEXPRESSION_
#define _H_IRISINSTANTVALUEEXPRESSION_
#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"
class IrisInstantValueExpression :
	public IrisExpression
{
protected:
	IrisInstantValueType m_eType = IrisInstantValueType::Nil;

public:

	virtual bool Generate();

	IrisInstantValueExpression(IrisInstantValueType eType);
	virtual ~IrisInstantValueExpression();
};

#endif