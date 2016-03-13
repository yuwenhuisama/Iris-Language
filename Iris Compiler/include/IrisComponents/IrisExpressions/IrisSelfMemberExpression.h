#ifndef _H_IRISSELFMEMBEREXPRESSION_
#define _H_IRISSELFMEMBEREXPRESSION_
#include "IrisExpression.h"

class IrisIdentifier;
class IrisSelfMemberExpression :
	public IrisExpression
{
protected:
	IrisIdentifier* m_pProperty = nullptr;

public:

	virtual bool Generate();
	virtual bool LeftValue(IrisAMType & eType, IR_DWORD & bIndex);

	IrisSelfMemberExpression(IrisIdentifier* pProperty);
	virtual ~IrisSelfMemberExpression();
};

#endif