#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisValidator/IrisAbstractExpressionValidateVisitor.h"


IrisExpression::IrisExpression()
{
}

bool IrisExpression::LeftValue(IrisAMType & eType, IR_DWORD & bIndex)
{
	return false;
}

IrisExpression::~IrisExpression()
{
}

bool IrisExpression::Accept(IrisAbstractExpressionValidateVisitor * pVisitor)
{
	return pVisitor->Visit(this);
}

bool IrisExpression::Validate()
{
	return true;
}
