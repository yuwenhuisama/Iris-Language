#ifndef _H_IRISEXPRESSIONVALIDATEVISITOR_
#define _H_IRISEXPRESSIONVALIDATEVISITOR_

#include "IrisValidator/IrisAbstractExpressionValidateVisitor.h"

class IrisExpressionValidateVisitor : public IrisAbstractExpressionValidateVisitor
{
public:
	IrisExpressionValidateVisitor();
	~IrisExpressionValidateVisitor();

	// Í¨¹ý IrisAbstractExpressionValidateVisitor ¼Ì³Ð
	virtual bool Visit(IrisExpression * pExpression) override;
};

#endif