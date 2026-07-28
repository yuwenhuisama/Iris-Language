#ifndef _H_IIRISSTATMENTVALIDATOR_
#define _H_IIRISSTATMENTVALIDATOR_

class IrisAbstractStatementValidateVisitor;
class IIrisStatementValidator {
public:
	virtual bool Accept(IrisAbstractStatementValidateVisitor* pVisitor) = 0;
	virtual bool Validate() = 0;
};

#endif
