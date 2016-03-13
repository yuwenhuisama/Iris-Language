#ifndef _H_IRISEXPRESSION_
#define _H_IRISEXPRESSION_
#include "IrisComponents/IrisVirtualCodeStructures.h"
class IrisExpression
{
protected:
	int m_nLineNumber = 0;
	bool m_bSelfExpressionFlag = false;

public:

	inline void SetLineNumber(unsigned int nLineNumber) { m_nLineNumber = nLineNumber; }
	inline unsigned int IrisExpression::GetLineNumber() { return m_nLineNumber; }

	IrisExpression();

	virtual bool LeftValue(IrisAMType& eType, IR_DWORD& bIndex);

	virtual bool Generate() = 0;
	virtual ~IrisExpression() = 0;

	friend class IrisMemberExpression;
};

#endif