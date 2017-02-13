#ifndef _H_IIRISEXPRESSIONVALIDATOR_
#define _H_IIRISEXPRESSIONVALIDATOR_

class IrisAbstractExpressionValidateVisitor;
class IIrisExpressionValidator {
public:
	virtual bool Accept(IrisAbstractExpressionValidateVisitor* pVisitor) = 0;
	virtual bool Validate() = 0;
};

#endif