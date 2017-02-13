#include "IrisExpressionValidateVisitor.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"


IrisExpressionValidateVisitor::IrisExpressionValidateVisitor()
{
}


IrisExpressionValidateVisitor::~IrisExpressionValidateVisitor()
{
}

bool IrisExpressionValidateVisitor::Visit(IrisExpression * pExpression)
{
	return pExpression->Validate();
}
