#ifndef _H_IRISCONDITIONIFSTATEMENT
#define _H_IRISCONDITIONIFSTATEMENT
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisExpression;
class IrisBlock;
class IrisElseIf;
class IrisConditionIfStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pCondition = nullptr;
	IrisBlock* m_pBlock = nullptr;
	IrisList<IrisElseIf*>* m_pIrisElseIf = nullptr;
	IrisBlock* m_pElseBlock = nullptr;

public:

	virtual bool Generate();

	IrisConditionIfStatement(IrisExpression* pCondition, IrisBlock* pBlock, IrisList<IrisElseIf*>* pIrisElseIf, IrisBlock* pElseBlock);
	virtual ~IrisConditionIfStatement();

	virtual bool Validate() override;
};

#endif