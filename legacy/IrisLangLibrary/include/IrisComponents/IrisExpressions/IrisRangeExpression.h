#ifndef _H_IRISRANGEEXPRESSION_
#define _H_IRISRANGEEXPRESSION_

#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"

class IrisRangeExpression :
	public IrisExpression

{
protected:
	IrisRangeType m_eType = IrisRangeType::LeftClosedAndRightClosed;
	IrisExpression* m_pLeftExpression = nullptr;
	IrisExpression* m_pRightExpression = nullptr;

public:

	virtual bool Generate();

	IrisRangeExpression(IrisRangeType eType, IrisExpression* pLeftExpression, IrisExpression* pRightExpression);
	~IrisRangeExpression();

	virtual bool Validate() override;
};

#endif