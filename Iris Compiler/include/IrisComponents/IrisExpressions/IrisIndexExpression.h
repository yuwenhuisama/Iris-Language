#ifndef _H_IRISINDEXEXPRESSION_
#define _H_IRISINDEXEXPRESSION_
#include "IrisExpression.h"
class IrisIndexExpression :
	public IrisExpression
{
protected:
	IrisExpression* m_pTarget = nullptr;
	IrisExpression* m_pIndexer = nullptr;

public:

	virtual bool Generate();
	virtual bool LeftValue(IrisAMType & eType, IR_DWORD & bIndex);

	IrisIndexExpression(IrisExpression* pTarget, IrisExpression* pIndexer);
	virtual ~IrisIndexExpression();
};

#endif