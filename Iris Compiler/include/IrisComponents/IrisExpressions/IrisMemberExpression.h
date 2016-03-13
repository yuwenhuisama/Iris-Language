#ifndef _H_IRISMEMBEREXPRESSION_
#define _H_IRISMEMBEREXPRESSION_
#include "IrisExpression.h"

class IrisIdentifier;
class IrisMemberExpression :
	public IrisExpression
{
protected:
	IrisExpression* m_pCaller = nullptr;
	IrisIdentifier* m_pProperty = nullptr;

public:

	virtual bool Generate();
	virtual bool LeftValue(IrisAMType & eType, IR_DWORD & bIndex);

	IrisMemberExpression(IrisExpression* pCaller, IrisIdentifier* pPropery);
	virtual ~IrisMemberExpression();
};

#endif