#ifndef _H_IRISBINARYEXPRESSION_
#define _H_IRISBINARYEXPRESSION_
#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"

class IrisBinaryExpression :
	public IrisExpression
{
protected:
	IrisExpression* m_pLeftExpression = nullptr;
	IrisExpression* m_pRightExpression = nullptr;
	IrisBinaryExpressionType m_eType = IrisBinaryExpressionType::Assign;

protected:
	bool OperateGenerate(unsigned int nOperatorIndex);
	bool AssignGenerate();
	bool OperateAssignGenerate(unsigned int nOperatorIndex);

public:

	virtual bool Generate();
	IrisBinaryExpression(IrisBinaryExpressionType eType, IrisExpression* pLeftExpression, IrisExpression* pRightExpression);
	virtual ~IrisBinaryExpression();

	virtual bool Validate() override;
};

#endif