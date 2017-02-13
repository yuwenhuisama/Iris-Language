#ifndef _H_IRISEXPRESSION_
#define _H_IRISEXPRESSION_
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisValidator/IIrisExpressionValidator.h"

class IrisExpression : public IIrisExpressionValidator
{
protected:
	int m_nLineNumber = 0;

public:

	inline void SetLineNumber(unsigned int nLineNumber) { m_nLineNumber = nLineNumber; }
	inline unsigned int IrisExpression::GetLineNumber() { return m_nLineNumber; }

	IrisExpression();

	virtual bool LeftValue(IrisAMType& eType, IR_DWORD& bIndex);

	virtual bool Generate() = 0;
	virtual ~IrisExpression() = 0;

	friend class IrisMemberExpression;

	// Í¨¹ý IIrisStatementValidator ¼Ì³Ð
	virtual bool Accept(IrisAbstractExpressionValidateVisitor * pVisitor) override;
	virtual bool Validate() override;
};

#endif