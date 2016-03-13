#ifndef _H_IRISUNARYEXPRESSION_
#define _H_IRISUNARYEXPRESSION_

#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"
class IrisUnaryExpression :
	public IrisExpression
{
protected:
	IrisUnaryExpressionType m_eType = IrisUnaryExpressionType::BitNot;
	IrisExpression* m_pExpression = nullptr;

public:

	virtual bool Generate();

	IrisUnaryExpression(IrisUnaryExpressionType eType, IrisExpression* pExpression);
	virtual ~IrisUnaryExpression();
};

#endif