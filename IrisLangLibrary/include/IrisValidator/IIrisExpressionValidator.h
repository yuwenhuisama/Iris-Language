#ifndef _H_IIRISEXPRESSIONVALIDATOR_
#define _H_IIRISEXPRESSIONVALIDATOR_

class IrisAbstractExpressionValidateVisitor;
class IIrisExpressionValidator {
protected:
	bool m_bValidLeftValue = false;

public:
	virtual bool Accept(IrisAbstractExpressionValidateVisitor* pVisitor) = 0;
	virtual bool Validate() = 0;

	bool ValidLeftValue() { return m_bValidLeftValue; }
};

#endif