#ifndef _H_IRISABSTRACTEXPRESSIONVALIDATEVISITOR_
#define _H_IRISABSTRACTEXPRESSIONVALIDATEVISITOR_

class IrisExpression;
class IrisAbstractExpressionValidateVisitor {
public:
	virtual bool Visit(IrisExpression* pExpression) = 0;
};

#endif // !_H_IRISABSTRACTEXPRESSIONVALIDATEVISITOR_
